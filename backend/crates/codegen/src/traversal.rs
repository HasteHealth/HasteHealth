use haste_fhir_model::r4::generated::{resources::StructureDefinition, types::ElementDefinition};

/// Returns the indices of the direct child elements of the element at `index`.
///
/// # Errors
///
/// Returns an error if:
/// - `index` is out of bounds.
/// - An element does not contain a path.
pub fn ele_index_to_child_indices(
    elements: &[ElementDefinition],
    index: usize,
) -> Result<Vec<usize>, String> {
    let parent = elements
        .get(index)
        .ok_or_else(|| format!("Index {index} out of bounds"))?;

    let parent_path: &str = parent.path.value.as_deref().ok_or("Element has no path")?;

    let depth = parent_path.matches('.').count();

    let mut children_indices = Vec::new();

    for (cur_index, element) in elements.iter().enumerate().skip(index + 1) {
        let path = element.path.value.as_deref().ok_or("Not Found")?;
        let path_depth = path.matches('.').count();

        if path_depth <= depth {
            break;
        }

        // A direct child has exactly one more path segment than its parent, with the
        // parent's path as a strict prefix followed by a `.` separator - equivalent to
        // the previous `^{parent}\.[^.]+$` regex without paying for regex compilation
        // on every call.
        if path_depth == depth + 1
            && path.len() > parent_path.len()
            && path.starts_with(parent_path)
            && path.as_bytes()[parent_path.len()] == b'.'
        {
            children_indices.push(cur_index);
        }
    }

    Ok(children_indices)
}

fn traversal_bottom_up_sd_elements<'a, F, V>(
    elements: &'a Vec<ElementDefinition>,
    index: usize,
    visitor_function: &mut F,
) -> Result<V, String>
where
    F: FnMut(&'a ElementDefinition, Vec<V>, usize) -> V,
{
    let child_indices = ele_index_to_child_indices(elements.as_slice(), index)?;

    let child_traversal_values: Vec<V> = child_indices
        .iter()
        .map(|&child_index| {
            traversal_bottom_up_sd_elements(elements, child_index, visitor_function)
        })
        .collect::<Result<Vec<V>, String>>()?;

    Ok(visitor_function(
        &elements[index],
        child_traversal_values,
        index,
    ))
}

/// Traverses the elements of a [`StructureDefinition`] in bottom-up order.
///
/// The visitor function is called after all child elements have been traversed.
///
/// # Errors
///
/// Returns an error if the [`StructureDefinition`] does not contain a snapshot.
pub fn traversal<'a, F, V>(sd: &'a StructureDefinition, visitor: &mut F) -> Result<V, String>
where
    F: FnMut(&'a ElementDefinition, Vec<V>, usize) -> V,
{
    let elements = &sd
        .snapshot
        .as_ref()
        .ok_or("StructureDefinition has no snapshot")?
        .element;

    traversal_bottom_up_sd_elements(elements, 0, visitor)
}

#[cfg(test)]
mod tests {

    use haste_fhir_model::r4::generated::resources::{Bundle, Resource};

    use super::*;

    #[test]
    fn test_traversal() {
        let bundle = serde_json::from_str::<Bundle>(
            &std::fs::read_to_string(
                "../artifacts/artifacts/r4/hl7/minified/profiles-resources.min.json",
            )
            .unwrap(),
        )
        .unwrap();

        let sds: Vec<&StructureDefinition> = bundle
            .entry
            .as_ref()
            .unwrap()
            .iter()
            .filter_map(
                |e| match e.resource.as_ref().map(std::convert::AsRef::as_ref) {
                    Some(Resource::StructureDefinition(sd)) => Some(sd),
                    _ => None,
                },
            )
            .collect();

        let mut visitor =
            |element: &ElementDefinition, children: Vec<String>, _index: usize| -> String {
                let path: String = element.path.value.as_ref().unwrap().clone();
                children.join("\n") + "\n" + &path
            };

        println!("StructureDefinitions: {}", sds.len());

        for sd in sds {
            let result = traversal(sd, &mut visitor);

            println!("Result: {result:?}");
        }
    }
}
