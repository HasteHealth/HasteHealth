pub fn escape_field(field: &str) -> String {
    field.replace("~", "~0").replace("/", "~1")
}

pub fn unescape_field(field: &str) -> String {
    field.replace("~1", "/").replace("~0", "~")
}
