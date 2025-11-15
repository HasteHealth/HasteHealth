use haste_fhir_model::r4::generated::{resources::StructureDefinition, types::ElementDefinition};
use regex::Regex;

fn ele_index_to_child_indices(
    elements: &[Box<ElementDefinition>],
    index: usize,
) -> Result<Vec<usize>, String> {
    let parent = elements
        .get(index)
        .ok_or_else(|| format!("Index {} out of bounds", index))?;

    let parent_path: String = parent
        .path
        .value
        .as_ref()
        .ok_or("Element has no path")?
        .to_string();

    let depth = parent_path.matches('.').count();
    let parent_path_escaped = parent_path.replace('.', "\\.");
    let child_regex = Regex::new(&format!("^{}\\.[^.]+$", parent_path_escaped))
        .map_err(|e| format!("Failed to compile regex: {}", e))?;

    let mut cur_index = index + 1;
    let mut children_indices = Vec::new();

    while cur_index < elements.len()
        && let path = elements[cur_index]
            .path
            .value
            .as_ref()
            .ok_or("Not Found")?
            .to_owned()
        && path.matches('.').count() > depth
    {
        if child_regex.is_match(&path) {
            children_indices.push(cur_index);
        }
        cur_index += 1;
    }

    Ok(children_indices)
}

fn traversal_bottom_up_sd_elements<'a, F, V>(
    elements: &'a Vec<Box<ElementDefinition>>,
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
        let bundle = haste_fhir_serialization_json::from_str::<Bundle>(
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
            .filter_map(|e| match e.resource.as_ref().map(|r| r.as_ref()) {
                Some(Resource::StructureDefinition(sd)) => Some(sd),
                _ => None,
            })
            .collect();

        let mut visitor =
            |element: &ElementDefinition, children: Vec<String>, _index: usize| -> String {
                let path: String = element.path.value.as_ref().unwrap().to_string();
                let result = children.join("\n") + "\n" + &path;
                result
            };

        println!("StructureDefinitions: {}", sds.len());

        for sd in sds {
            let result = traversal(sd, &mut visitor);

            println!("Result: {:?}", result);
        }
    }
}
