use crate::{ProjectId, ResourceId, TenantId, VersionId};
use oxidized_reflect::MetaValue;

impl MetaValue for TenantId {
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
        "http://hl7.org/fhirpath/System.TenantId"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for ProjectId {
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
        "http://hl7.org/fhirpath/System.ProjectId"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for ResourceId {
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
        "http://hl7.org/fhirpath/System.ResourceId"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for VersionId {
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
        "http://hl7.org/fhirpath/System.VersionId"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}
