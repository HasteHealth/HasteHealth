use crate::r4::datetime::{Date, DateTime, Instant, Time};
use oxidized_reflect::MetaValue;
use std::any::Any;

impl MetaValue for Time {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn typename(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.Time"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for DateTime {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn typename(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.DateTime"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for Date {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn typename(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.Date"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for Instant {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn typename(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.Instant"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}
