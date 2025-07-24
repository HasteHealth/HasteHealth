use std::{any::Any, fmt::Debug};

pub trait MetaValue: Any + Debug {
    fn fields(&self) -> Vec<&'static str>;

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue>;
    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue>;

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a dyn MetaValue>;
    fn get_index_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut dyn MetaValue>;

    fn flatten(&self) -> Vec<&dyn MetaValue>;

    fn as_any(&self) -> &dyn Any;

    fn typename(&self) -> &'static str;
}
