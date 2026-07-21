use std::{any::Any, fmt::Debug};

pub trait MetaValue: Any + Debug + Send + Sync {
    fn fields(&self) -> Vec<&'static str>;

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue>;
    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue>;

    fn get_index(&self, index: usize) -> Option<&dyn MetaValue>;
    fn get_index_mut(&mut self, index: usize) -> Option<&mut dyn MetaValue>;

    fn flatten(&self) -> Vec<&dyn MetaValue>;

    fn as_any(&self) -> &dyn Any;

    fn fhir_type(&self) -> &'static str;

    fn is_many(&self) -> bool;
}
