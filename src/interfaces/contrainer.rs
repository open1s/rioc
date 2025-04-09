pub trait Provider: Send + Sync + 'static {
    fn instantiate(&self);
}


pub trait Container {
    fn resolve<T: Provider>(&self) -> Option<&T>;
    fn register<T: Provider>(&mut self, instance: T);
}