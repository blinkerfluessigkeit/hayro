use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use hayro_syntax::object::ObjectIdentifier;

#[derive(Clone)]
pub struct Cache(Arc<Mutex<HashMap<ObjectIdentifier, Option<Box<dyn Any>>>>>);

impl Cache {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
    
    pub fn get_or_insert_with<T: Clone + 'static>(
        &self,
        id: ObjectIdentifier,
        f: impl FnOnce() -> Option<T>
    ) -> Option<T> {
        self.0.lock().unwrap().entry(id).or_insert_with(|| f().map(|val| Box::new(val) as Box<dyn Any>))
            .as_ref().and_then(|val| val.downcast_ref::<T>().map(|val| val.clone()))
        
    }
}