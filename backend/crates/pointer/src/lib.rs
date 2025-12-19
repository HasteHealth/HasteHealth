use haste_reflect::MetaValue;
pub struct Pointer<'a> {
    value: Option<&'a dyn MetaValue>,
    path: String,
}

pub enum Key {
    Field(String),
    Index(usize),
}

fn path_descend(path: &str, key: &str) -> String {
    format!("{}/{}", path, key)
}

impl<'a> Pointer<'a> {
    pub fn path(&self) -> &str {
        self.path.as_str()
    }
    pub fn value(&self) -> Option<&'a dyn MetaValue> {
        self.value
    }
    pub fn root(value: &'a dyn MetaValue) -> Self {
        Pointer {
            value: Some(value),
            path: "".to_string(),
        }
    }

    pub fn descend(&self, key: &Key) -> Pointer<'a> {
        match key {
            Key::Field(field) => Self {
                path: path_descend(self.path.as_str(), field),
                value: self.value.and_then(|v| v.get_field(field)),
            },
            Key::Index(index) => Self {
                path: path_descend(self.path.as_str(), index.to_string().as_str()),
                value: self.value.and_then(|v| v.get_index(*index)),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::Patient, types::FHIRString, types::HumanName,
    };

    #[test]
    fn test_pointer_descend() {
        let patient = Patient {
            id: Some("patient-1".to_string()),
            name: Some(vec![Box::new(HumanName {
                family: Some(Box::new(FHIRString {
                    value: Some("Doe".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            })]),
            ..Default::default()
        };

        let pointer = Pointer::root(&patient);
        let pointer = pointer.descend(&Key::Field("name".to_string()));
        assert_eq!(pointer.path(), "/name");
        let pointer = pointer.descend(&Key::Index(0));
        assert_eq!(pointer.path(), "/name/0");
        let pointer = pointer
            .descend(&Key::Field("family".to_string()))
            .descend(&Key::Field("value".to_string()));

        assert_eq!(pointer.path(), "/name/0/family/value");
        assert_eq!(
            pointer
                .value()
                .and_then(|v| v.as_any().downcast_ref::<String>()),
            Some(&"Doe".to_string())
        );
    }
}
