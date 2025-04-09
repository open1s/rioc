
use std::any::Any;
use std::collections::HashMap;
use crate::interfaces::container::{Container, Provider};

pub struct BasicContainer {
    instances: HashMap<std::any::TypeId, Box<dyn Any + Send + Sync>>,
}

impl BasicContainer {
    pub fn new() -> Self {
        BasicContainer {
            instances: HashMap::new(),
        }
    }
}

impl Container for BasicContainer {
    fn resolve<T: 'static>(&self) -> Option<&T> {
        let type_id = std::any::TypeId::of::<T>();
        self.instances.get(&type_id)
            .and_then(|instance| instance.downcast_ref::<T>())
    }

    fn register<P: Provider + 'static>(&mut self, provider: P) {
        let instance = provider.instantiate();
        let type_id = (*instance).type_id();
        self.instances.insert(type_id, instance);
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

    struct TestProvider(Arc<String>);

    impl Provider for TestProvider {
        type Output = Arc<String>;

        fn instantiate(&self) -> Box<Self::Output> {
            Box::new(Arc::clone(&self.0))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_container() {
        let mut container = BasicContainer::new();
        let test_value = Arc::new(String::from("test"));
        container.register(TestProvider(Arc::clone(&test_value)));

        let resolved: Option<&Arc<String>> = container.resolve();
        assert!(resolved.is_some(), "Failed to resolve Arc<String>");
        if let Some(value) = resolved {
            assert_eq!(value.as_str(), "test");
        }
    }

    #[test]
    fn test_multiple_registrations() {
        let mut container = BasicContainer::new();
        
        // Register a string
        container.register(TestProvider(Arc::new(String::from("test"))));
        
        // Verify we can resolve it
        let resolved: Option<&Arc<String>> = container.resolve();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().as_str(), "test");
    }
}