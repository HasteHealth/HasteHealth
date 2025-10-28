use crate::traits::MetaValue;
use std::any::Any;

/**
 * 067  public static final String FP_String = "http://hl7.org/fhirpath/System.String";
 * 068  public static final String FP_Boolean = "http://hl7.org/fhirpath/System.Boolean";
 * 069  public static final String FP_Integer = "http://hl7.org/fhirpath/System.Integer";
 * 070  public static final String FP_Decimal = "http://hl7.org/fhirpath/System.Decimal";
 * 071  public static final String FP_Quantity = "http://hl7.org/fhirpath/System.Quantity";
 * 072  public static final String FP_DateTime = "http://hl7.org/fhirpath/System.DateTime";
 * 073  public static final String FP_Time = "http://hl7.org/fhirpath/System.Time";
 */

impl MetaValue for i64 {
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
        "http://hl7.org/fhirpath/System.Integer"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for u64 {
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
        "http://hl7.org/fhirpath/System.Integer"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for f64 {
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
        "http://hl7.org/fhirpath/System.Decimal"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for bool {
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
        "http://hl7.org/fhirpath/System.Boolean"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for String {
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
        "http://hl7.org/fhirpath/System.String"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl MetaValue for &'static str {
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
        "http://hl7.org/fhirpath/System.String"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }
}

impl<T> MetaValue for Vec<T>
where
    T: MetaValue,
{
    fn fields(&self) -> Vec<&'static str> {
        if let Some(first) = self.first() {
            first.fields()
        } else {
            vec![]
        }
    }

    fn get_field<'a>(&'a self, _field: &str) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a dyn MetaValue> {
        if self.get(index).is_some() {
            let k: &dyn MetaValue = &self[index];
            Some(k)
        } else {
            None
        }
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut dyn MetaValue> {
        if self.get(index).is_some() {
            let k: &mut dyn MetaValue = &mut self[index];
            Some(k)
        } else {
            None
        }
    }

    fn typename(&self) -> &'static str {
        if let Some(first) = self.first() {
            first.typename()
        } else {
            ""
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        self.iter().flat_map(|item| item.flatten()).collect()
    }
}

impl<T> MetaValue for Box<T>
where
    T: MetaValue,
{
    fn fields(&self) -> Vec<&'static str> {
        self.as_ref().fields()
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        self.as_ref().get_field(field)
    }

    fn typename(&self) -> &'static str {
        self.as_ref().typename()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        self.as_ref().flatten()
    }

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a dyn MetaValue> {
        self.as_ref().get_index(index)
    }

    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
        self.as_mut().get_field_mut(field)
    }

    fn get_index_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut dyn MetaValue> {
        self.as_mut().get_index_mut(index)
    }
}
