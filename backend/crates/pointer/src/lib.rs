use haste_reflect::MetaValue;
pub struct Pointer<'a, T: MetaValue, U: MetaValue> {
    root: &'a T,
    value: &'a U,
    path: String,
}

pub enum Key {
    Field(String),
    Index(usize),
}

fn path_descend(path: &str, key: &str) -> String {
    format!("{}/{}", path, key)
}

pub fn pointer<'a, T: MetaValue>(value: &'a T) -> Pointer<'a, T, T> {
    Pointer {
        root: value,
        value: value,
        path: "".to_string(),
    }
}

impl<'a, Root: MetaValue, U: MetaValue> Pointer<'a, Root, U> {
    pub fn new(value: &'a Root) -> Pointer<'a, Root, Root> {
        Pointer {
            root: value,
            value: value,
            path: "".to_string(),
        }
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn value(&self) -> Option<&'a U> {
        self.value.as_any().downcast_ref::<U>()
    }

    pub fn descend<Child: MetaValue>(&'a self, key: &Key) -> Option<Pointer<'a, Root, Child>> {
        match key {
            Key::Field(field) => self
                .value
                .get_field(field)
                .and_then(|v| v.as_any().downcast_ref::<Child>())
                .map(|child| Pointer {
                    root: self.root,
                    value: child,
                    path: path_descend(self.path.as_str(), field.as_str()),
                }),
            Key::Index(index) => self
                .value
                .get_index(*index)
                .and_then(|v| v.as_any().downcast_ref::<Child>())
                .map(|child| Pointer {
                    root: self.root,
                    value: child,
                    path: path_descend(self.path.as_str(), index.to_string().as_str()),
                }),
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

        let pointer = Pointer::<Patient, Patient>::new(&patient);
        let pointer = pointer
            .descend::<Vec<Box<HumanName>>>(&Key::Field("name".to_string()))
            .unwrap();
        assert_eq!(pointer.path(), "/name");
        let pointer = pointer.descend::<Box<HumanName>>(&Key::Index(0)).unwrap();
        assert_eq!(pointer.path(), "/name/0");
        let pointer = pointer
            .descend::<Box<FHIRString>>(&Key::Field("family".to_string()))
            .unwrap();
        let pointer = pointer
            .descend::<String>(&Key::Field("value".to_string()))
            .unwrap();

        assert_eq!(pointer.path(), "/name/0/family/value");
        assert_eq!(pointer.value(), Some(&"Doe".to_string()));
    }
}
