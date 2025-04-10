# Rioc Derive Macros

This crate provides derive macros for the Rioc dependency injection framework.

## Provider Macro

The `#[derive(Provider)]` macro generates implementation code for dependency injection.

### Basic Usage

```rust
use rioc::{Provider, Container};

// Define a dependency
struct MyDependency;

// Define a service that depends on MyDependency
#[derive(Provider)]
struct MyService {
    #[inject(name = "my_dependency")]
    dependency: MyDependency,
}

fn main() {
    // Create a container
    let mut container = Container::new();
    
    // Register the dependency
    container.register("my_dependency", MyDependency);
    
    // Resolve the service with dependencies injected
    let service: MyService = container.resolve();
    
    // Now you can use the service
    println!("Service created successfully!");
}
```

### Generic Types

```rust
use rioc::Provider;

#[derive(Provider)]
struct GenericService<T> {
    dependency: T,
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rioc-derives = { path = "../derives" }
```