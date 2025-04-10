use derives::IProvider;

#[derive(IProvider)]
struct SimpleService {
    #[inject(name = "dependency")]
    dependency: String,
}

// #[derive(Default)]
// struct GenericService<T> {
//     #[inject(name = "dependency")]
//     dependency: T,
// }

// #[test]
// fn test_simple_service() {
//     let container = rioc::containers::basic::BasicContainer::new();
//     container.register_by_name::<String>("dependency", "test_value".to_string());
    
//     let service = <SimpleService as Provider>::resolve(&container);
//     assert_eq!(service.dependency, "test_value");
// }

// #[test]
// fn test_generic_service() {
//     let container = rioc::containers::basic::BasicContainer::new();
//     container.register_by_name::<i32>("dependency", 42);
    
//     let service = <GenericService<i32> as Provider>::resolve(&container);
//     assert_eq!(service.dependency, 42);
// }