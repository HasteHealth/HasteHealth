use std::sync::Arc;

use haste_reflect::MetaValue;
use minijinja::{
    Value,
    value::{Enumerator, Object, ObjectRepr},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ObjectConverter<'a>(&'a dyn MetaValue);

impl<'a> Object for ObjectConverter<'a> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(field) = key.as_str()
            && let Some(value) = self.0.get_field(field)
        {
            let k = unsafe {
                let value = std::mem::transmute::<&dyn MetaValue, &'static dyn MetaValue>(value);
                Some(Value::from_dyn_object(Arc::new(ObjectConverter(value))))
            };

            return k;
        } else if let Some(index) = key.as_usize()
            && let Some(value) = self.0.get_index(index)
        {
            let k = unsafe {
                let value = std::mem::transmute::<&dyn MetaValue, &'static dyn MetaValue>(value);
                Some(Value::from_dyn_object(Arc::new(ObjectConverter(value))))
            };

            return k;
        } else {
            None
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::NonEnumerable
    }
}
