use regex::Regex;
use serde_json::{Value, from_value};

fn ele_index_to_child_indices(elements: &[Value], index: usize) -> Result<Vec<usize>, String> {
    let parent = elements
        .get(index)
        .ok_or_else(|| format!("Index {} out of bounds", index))?;

    let parent_path: String = parent
        .get("path")
        .ok_or("Element has no path")?
        .as_str()
        .ok_or("Path is not a string")?
        .to_string();

    let depth = parent_path.matches('.').count();
    let parent_path_escaped = parent_path.replace('.', "\\.");
    let child_regex = Regex::new(&format!("^{}\\.[^.]+$", parent_path_escaped))
        .map_err(|e| format!("Failed to compile regex: {}", e))?;

    let mut cur_index = index + 1;
    let mut children_indices = Vec::new();

    while cur_index < elements.len()
        && let path = from_value::<String>(
            elements[cur_index]
                .get("path")
                .ok_or("Not Found")?
                .to_owned(),
        )
        .map_err(|_| "Failed to serialize")?
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
    elements: &'a [Value],
    index: usize,
    visitor_function: &mut F,
) -> Result<V, String>
where
    F: FnMut(&'a Value, Vec<V>, usize) -> V,
{
    let child_indices = ele_index_to_child_indices(elements, index)?;

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

pub fn traversal<'a, F, V>(sd: &'a Value, visitor: &mut F) -> Result<V, String>
where
    F: FnMut(&'a Value, Vec<V>, usize) -> V,
{
    let elements = sd
        .get("snapshot")
        .ok_or("StructureDefinition has no snapshot")?
        .get("element")
        .ok_or("StructureDefinition has no elements")?;

    if let Some(elements) = elements.as_array() {
        traversal_bottom_up_sd_elements(elements, 0, visitor)
    } else {
        return Err("Elements is not an array".to_string());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_traversal() {
        let data = serde_json::from_str::<Value>(
            &std::fs::read_to_string("../artifacts/r4/hl7/profiles-resources.json").unwrap(),
        )
        .unwrap();

        let sds: Vec<&Value> = data
            .get("entry")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                let resource_type = e
                    .get("resource")
                    .and_then(|r| r.get("resourceType"))
                    .and_then(|rt| rt.as_str())
                    .unwrap_or("");

                resource_type == "StructureDefinition"
            })
            .map(|e| e.get("resource").unwrap())
            .collect();

        let mut visitor = |element: &Value, children: Vec<String>, _index: usize| -> String {
            let path: String = element
                .get("path")
                .and_then(|r| from_value::<String>(r.to_owned()).ok())
                .unwrap_or("".to_string());

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
