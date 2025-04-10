use derives::IProvider;
use rioc::Container;
use rioc::Provider;

#[derive(IProvider,Debug, Clone)]
struct SimpleService {
    #[inject(name = "dependency")]
    dependency: String,
}

impl SimpleService {
    fn new() -> Self {
        SimpleService { dependency: "hello".to_string() }
    }
}

#[derive(Default,IProvider,Debug, Clone)]
struct GenericService<T: Clone + Sync + Send> {
    #[inject(name = "dependency")]
    dependency: T,
}

#[test]
fn test_simple_service() {
    let container = rioc::containers::basic::BasicContainer::new();
    container.register_by_name("dependency".to_string(), SimpleService::new());
    
    let service: Option<Box<SimpleService>> = container.resolve_by_name("dependency");
    assert!(service.is_some());
    if let Some(service) = service {
        println!("@{:#?}", service);
    }

    let service: Option<Box<SimpleService>> = container.resolve_by_name("dependency");
    assert!(service.is_some());
    if let Some(service) = service {
        println!("@@{:#?}", service);
    }
}

#[test]
fn test_generic_service() {
    let container = rioc::containers::basic::BasicContainer::new();
    container.register_by_name("dependency".to_string(), GenericService { dependency: 42 });


    let service: Option<Box<GenericService<i32>>> = container.resolve_by_name("dependency");
    assert!(service.is_some());
    if let Some(service) = service {
        println!("@@{:#?}", service);
    }
}