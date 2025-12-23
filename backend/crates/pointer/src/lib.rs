use haste_reflect::MetaValue;
use std::sync::Arc;

#[derive(Clone)]
struct ChildPointer<U>(*const U);

unsafe impl<U> Send for ChildPointer<U> {}
unsafe impl<U> Sync for ChildPointer<U> {}

#[derive(Clone)]
pub struct Pointer<T: MetaValue, U: MetaValue> {
    root: Arc<T>,
    value: ChildPointer<U>,
    path: String,
}

pub enum Key {
    Field(String),
    Index(usize),
}

fn path_descend(path: &str, key: &str) -> String {
    format!("{}/{}", path, key)
}

impl<'a, Root: MetaValue, U: MetaValue> Pointer<Root, U> {
    pub fn new(value: Arc<Root>) -> Pointer<Root, Root> {
        Pointer {
            value: ChildPointer(&*value.as_ref() as *const Root),
            root: value,
            path: "".to_string(),
        }
    }

    pub fn root(&self) -> Pointer<Root, Root> {
        Pointer {
            value: ChildPointer(&*self.root.as_ref() as *const Root),
            root: self.root.clone(),
            path: "".to_string(),
        }
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn value(&self) -> Option<&U> {
        let p = unsafe { (*self.value.0).as_any().downcast_ref::<U>() };

        p
    }

    pub fn descend<Child: MetaValue>(&'a self, key: &Key) -> Option<Pointer<Root, Child>> {
        match key {
            Key::Field(field) => self.value().and_then(|v| {
                v.get_field(field)
                    .and_then(|v| v.as_any().downcast_ref::<Child>())
                    .map(|child| Pointer {
                        root: self.root.clone(),
                        value: ChildPointer(&*child as *const Child),
                        path: path_descend(self.path.as_str(), field.as_str()),
                    })
            }),
            Key::Index(index) => self.value().and_then(|v| {
                v.get_index(*index)
                    .and_then(|v| v.as_any().downcast_ref::<Child>())
                    .map(|child| Pointer {
                        root: self.root.clone(),
                        value: ChildPointer(&*child as *const Child),
                        path: path_descend(self.path.as_str(), index.to_string().as_str()),
                    })
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
        let patient = Arc::new(Patient {
            id: Some("patient-1".to_string()),
            name: Some(vec![Box::new(HumanName {
                family: Some(Box::new(FHIRString {
                    value: Some("Doe".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            })]),
            ..Default::default()
        });

        let pointer = Pointer::<Patient, Patient>::new(patient);
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
