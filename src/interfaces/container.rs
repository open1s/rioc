
use std::any::Any;

pub trait Provider: Send + Sync {    
    fn instantiate<C: Container>(&self, container: &C) -> Box<Self>;
    fn as_any(&self) -> &dyn Any;
}

pub trait Container: Send + Sync {
    // Add Clone bound to T since we need to clone values
    fn resolve<T: 'static + Clone>(&self) -> Option<Box<T>>;
    fn resolve_by_name<T: 'static + Clone>(&self, name: &str) -> Option<Box<T>>;
    // Remove &mut requirement to support concurrent access
    fn register<P: Provider + 'static>(&self, provider: P);
    fn register_by_name<P: Provider + 'static>(&self,name: String, provider: P);
}