 use rioc::{injectable, provider};

 #[derive(Debug)]
 struct DependencyToProvide {
     value: i32,
 }

 #[derive(Debug)]
 struct SharedDependencyToProvide {
     value: i32,
 }

 #[derive(Debug)]
 #[injectable]
 struct Facade<'a>(DependencyToProvide, &'a SharedDependencyToProvide);

 #[derive(Debug)]
 #[provider]
 #[provide(DependencyToProvide, DependencyToProvide { value: 42 })]
 struct Provider {
     #[provide]
     shared: SharedDependencyToProvide
 }


fn main() {
    let provider = Provider { shared: SharedDependencyToProvide { value: 123 } };
    let facade: Facade = provider.provide();
    println!("Facade value: {:#?}", facade);
}
