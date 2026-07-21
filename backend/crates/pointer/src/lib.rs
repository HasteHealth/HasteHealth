use haste_reflect::MetaValue;
use std::{fmt::Display, sync::Arc};

mod escape;

#[derive(Debug, Clone)]
pub struct Path(String);

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Path {
    #[must_use]
    pub fn new() -> Self {
        Self(String::new())
    }
    #[must_use]
    pub fn descend(&self, field: &str) -> Self {
        Self(format!("{}/{}", self.0, escape::escape_field(field)))
    }
    #[must_use]
    pub fn ascend(&self) -> Option<(Self, Key)> {
        if self.0.is_empty() {
            return None;
        }

        // Find the last '/' separator from the end of the string
        match self.0.rfind('/') {
            // Case 1: Separator found (e.g., "a/b/c" -> parent: "a/b", field: "c")
            Some(idx) => {
                let parent_path = &self.0[..idx];
                let field = &self.0[idx + 1..];
                Some((
                    Path(parent_path.to_string()),
                    Key::parse(&escape::unescape_field(field)),
                ))
            }
            // Case 2: No separator found, the string is a single field (e.g., "a" -> parent: "", field: "a")
            None => Some((
                Path(String::new()),
                Key::parse(&escape::unescape_field(&self.0)),
            )),
        }
    }

    pub fn get<'a>(&self, value: &'a dyn MetaValue) -> Option<&'a dyn MetaValue> {
        let mut current = value;
        // Skip the first empty part from the leading '/'
        for part in self.0.split('/').skip(1) {
            let k = Key::parse(&escape::unescape_field(part));

            match k {
                Key::Field(field) => {
                    current = current.get_field(&field)?;
                }
                Key::Index(index) => {
                    current = current.get_index(index)?;
                }
            }
        }

        Some(current)
    }

    pub fn get_typed<'a, Type: MetaValue>(&self, value: &'a dyn MetaValue) -> Option<&'a Type> {
        let current = self.get(value)?;
        current.as_any().downcast_ref::<Type>()
    }
}

#[derive(Debug)]
pub enum Key {
    Field(String),
    Index(usize),
}

impl Key {
    #[must_use]
    pub fn parse(field: &str) -> Self {
        if let Ok(index) = field.parse::<usize>() {
            Key::Index(index)
        } else {
            Key::Field(field.to_string())
        }
    }
}

#[derive(Clone)]
struct ChildPointer<U>(*const U);

unsafe impl<U> Send for ChildPointer<U> {}
unsafe impl<U> Sync for ChildPointer<U> {}

#[derive(Clone)]
pub struct TypedPointer<T: MetaValue, U: MetaValue> {
    root: Arc<T>,
    value: ChildPointer<U>,
    path: Path,
}

impl<Root: MetaValue, U: MetaValue> TypedPointer<Root, U> {
    pub fn new(value: Arc<Root>) -> TypedPointer<Root, Root> {
        TypedPointer {
            value: ChildPointer(&raw const *value.as_ref()),
            root: value,
            path: Path::new(),
        }
    }

    #[must_use]
    pub fn root(&self) -> TypedPointer<Root, Root> {
        TypedPointer {
            value: ChildPointer(&raw const *self.root.as_ref()),
            root: self.root.clone(),
            path: Path::new(),
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        self.path.0.as_str()
    }

    #[must_use]
    pub fn value(&self) -> Option<&U> {
        unsafe { (*self.value.0).as_any().downcast_ref::<U>() }
    }

    #[must_use]
    pub fn descend<Child: MetaValue>(&self, field: &Key) -> Option<TypedPointer<Root, Child>> {
        match field {
            Key::Field(field) => self.value().and_then(|v| {
                v.get_field(field)
                    .and_then(|v| v.as_any().downcast_ref::<Child>())
                    .map(|child| TypedPointer {
                        root: self.root.clone(),
                        value: ChildPointer(&raw const *child),
                        path: self.path.descend(field),
                    })
            }),
            Key::Index(index) => self.value().and_then(|v| {
                v.get_index(*index)
                    .and_then(|v| v.as_any().downcast_ref::<Child>())
                    .map(|child| TypedPointer {
                        root: self.root.clone(),
                        value: ChildPointer(&raw const *child),
                        path: self.path.descend(&index.to_string()),
                    })
            }),
        }
    }

    #[must_use]
    pub fn ascend(&self) -> Option<(Path, Key)> {
        self.path.ascend()
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

        let pointer = TypedPointer::<Patient, Patient>::new(patient);
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

    #[test]
    fn test_path() {
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

        let path = Path::new()
            .descend("name")
            .descend("0")
            .descend("family")
            .descend("value");

        assert_eq!(path.0, "/name/0/family/value");
        let k = path.get_typed::<String>(patient.as_ref());

        assert_eq!(k, Some(&"Doe".to_string()));
    }
}
