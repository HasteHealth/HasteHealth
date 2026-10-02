use crate::r4::datetime::{Date, DateTime, Instant, Time};
use haste_reflect::MetaValue;
use std::any::Any;

impl MetaValue for Time {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index(&self, _index: usize) -> Option<&dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut(&mut self, _index: usize) -> Option<&mut dyn MetaValue> {
        None
    }

    fn fhir_type(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.Time"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
    fn is_many(&self) -> bool {
        false
    }
}

impl MetaValue for DateTime {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index(&self, _index: usize) -> Option<&dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut(&mut self, _index: usize) -> Option<&mut dyn MetaValue> {
        None
    }

    fn fhir_type(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.DateTime"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
    fn is_many(&self) -> bool {
        false
    }
}

impl MetaValue for Date {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index(&self, _index: usize) -> Option<&dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut(&mut self, _index: usize) -> Option<&mut dyn MetaValue> {
        None
    }

    fn fhir_type(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.Date"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
    fn is_many(&self) -> bool {
        false
    }
}

impl MetaValue for Instant {
    fn fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index(&self, _index: usize) -> Option<&dyn MetaValue> {
        None
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut(&mut self, _index: usize) -> Option<&mut dyn MetaValue> {
        None
    }

    fn fhir_type(&self) -> &'static str {
        "http://hl7.org/fhirpath/System.Instant"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
    fn is_many(&self) -> bool {
        false
    }
}
