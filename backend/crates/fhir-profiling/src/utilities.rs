/// Various utilities for working with FHIR profiles.

pub fn remove_type_on_path(path: &str) -> &str {
    let first_dot = path.find('.');
    // If first element this would be the entire path as no subfield.
    &path[first_dot.map(|i| i + 1).unwrap_or(path.len())..]
}
