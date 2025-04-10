# Rioc - Rust IOC/DI Framework

A lightweight Inversion of Control (IoC) and Dependency Injection (DI) framework for Rust applications.

## Features

- Dependency injection container
- Interface-based programming support
- Lightweight and fast
- Thread-safe implementation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rioc = "0.1.0"
```

## Basic Usage

```rust
use rioc::containers::Container;

// Define your interfaces and implementations
// ...

// Create a container and register your dependencies
let mut container = Container::new();
container.register::<dyn MyInterface, MyImplementation>();

// Resolve dependencies
let service: Box<dyn MyInterface> = container.resolve().unwrap();
```
## Documentation

Coming soon...

## License

MIT
