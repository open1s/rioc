use crate::interfaces::contrainer::{Container, Provider};

pub struct BasicContainer {
    instances: dashmap::DashMap<std::any::TypeId, Box<dyn Provider>>,
}

impl BasicContainer {
    pub fn new() -> Self {
        Self {
            instances: dashmap::DashMap::new(),
        }
    }
}


impl Container for BasicContainer {
    fn resolve<T: Provider>(&self) -> Option<&T> {
        let id = std::any::TypeId::of::<T>();
        let instance = self.instances.get(&id);
        let instance = instance.map(|provider| {
            let boxed = provider.value();
            &boxed
        });
        None
    }

    fn register<T: Provider>(&mut self, instance: T) where T: Provider + 'static{
        self.instances.insert(std::any::TypeId::of::<T>(), Box::new(instance));
    }
}