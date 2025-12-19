use haste_reflect::MetaValue;
pub struct Pointer<'a> {
    pub value: Option<&'a dyn MetaValue>,
    pub path: String,
}

fn path_descend(path: &str, key: &str) -> String {
    format!("{}/{}", path, key)
}

impl<'a> Pointer<'a> {
    pub fn root(value: &'a dyn MetaValue) -> Self {
        Pointer {
            value: Some(value),
            path: "/".to_string(),
        }
    }

    pub fn descend(&self, key: &str) -> Pointer<'a> {
        Self {
            path: path_descend(self.path.as_str(), key),
            value: self.value.and_then(|v| v.get_field(key)),
        }
    }
}
