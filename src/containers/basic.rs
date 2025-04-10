
use std::any::Any;
use dashmap::DashMap;
use crate::interfaces::container::{Container, Provider};

pub struct BasicContainer {
    by_type: DashMap<std::any::TypeId, Box<dyn Any + Send + Sync>>,
    by_name: DashMap<String, Box<dyn Any + Send + Sync>>,
}

impl BasicContainer {
    pub fn new() -> Self {
        BasicContainer {
            by_type: DashMap::new(),
            by_name: DashMap::new(),
        }
    }
}

impl Container for BasicContainer {
    fn resolve<T: 'static + Clone>(&self) -> Option<Box<T>> {
        let type_id = std::any::TypeId::of::<T>();
        self.by_type.get(&type_id)
            .and_then(|instance| {
                instance.value()
                    .downcast_ref::<T>()
                    .map(|value| Box::new(value.clone()))
            })
    }

    fn resolve_by_name<T:'static + Clone>(&self, name: &str) -> Option<Box<T>> {
        self.by_name.get(name)
            .and_then(|instance| {
                instance.value()
                    .downcast_ref::<T>()
                    .map(|value| Box::new(value.clone()))
            })
    }

    fn register<P: Provider + 'static>(&self, provider: P) {
        let instance = provider.instantiate(self);
        let type_id = (*instance).type_id();
        self.by_type.insert(type_id, instance);
    }

    fn register_by_name<P: Provider + 'static>(&self,name: String, provider: P) {
        let instance = provider.instantiate(self);
        self.by_name.insert(name, instance);
    }
}

impl Default for BasicContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[derive(Clone)]
    struct TestProvider(Arc<String>);

    impl Provider for TestProvider {
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn instantiate<C: Container>(&self, _c: &C) -> Box<Self> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_container() {
        let container = BasicContainer::new();
        let test_value = Arc::new(String::from("test"));
        container.register(TestProvider(Arc::clone(&test_value)));

        let resolved = container.resolve::<Arc<String>>();
        assert!(resolved.is_none());
        
        let resolved = container.resolve::<TestProvider>();
        assert!(resolved.is_some());
    }

    #[test]
    fn test_concurrent_access() {
        let container = Arc::new(BasicContainer::new());
        let mut handles = vec![];

        // Spawn multiple threads to test concurrent access
        for i in 0..10 {
            let container = Arc::clone(&container);
            let handle = thread::spawn(move || {
                let value = Arc::new(format!("test_{}", i));
                let provider = TestProvider(Arc::clone(&value));
                
                // Register and resolve in each thread
                container.register(provider.clone());
                
                // Allow some time for other threads
                thread::yield_now();
                
                // Try to resolve our value
                let resolved = container.resolve::<TestProvider>();
                assert!(resolved.is_some());
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_resolve() {
        let container = Arc::new(BasicContainer::new());
        let test_value = Arc::new(String::from("test"));
        container.register(TestProvider(Arc::clone(&test_value)));

        let mut handles = vec![];

        // Spawn multiple threads to test concurrent resolves
        for _ in 0..10 {
            let container = Arc::clone(&container);
            let handle = thread::spawn(move || {
                let resolved = container.resolve::<TestProvider>();
                assert!(resolved.is_some());
                assert_eq!(resolved.unwrap().0.as_str(), "test");
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }
}