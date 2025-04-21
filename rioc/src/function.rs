use core::fmt;


pub trait Service<Input,Output> {
    fn call(&self, input: Input) -> Output;
}

impl <'a,Input,Output,T> Service<Input,Output> for &'a T 
where T: ?Sized + Service<Input,Output>
{
    fn call(&self, req: Input) -> Output {
        (*self).call(req)
    }
}

pub fn service<F,Input,Output>(f: F) -> Function<F,Input,Output>
where F: Fn(Input) -> Output {
    Function::new(f)
}

pub struct Function<F,Input,Output>{
    func: F,
    _marker: std::marker::PhantomData<(Input,Output)>,
}

impl<F,Input,Output> Function<F,Input,Output> 
where F: Fn(Input) -> Output
{
    pub fn new(func: F) -> Self {
        Self { 
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F,Input,Output> fmt::Debug for Function<F,Input,Output> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl <F,Input,Output> Service<Input,Output> for Function<F,Input,Output>
where
    F: Fn(Input) -> Output,
{
    fn call(&self, req:Input) -> Output {
        (self.func)(req)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_service_fn() {
        let f = |x: i32| x + 1;
        let service = service(f);
        assert_eq!(service.call(1), 2);
    }

    #[test]
    fn layer_fn_has_useful_debug_impl() {
        struct WrappedService<S> {
            inner: S,
        }
        let layer = Arc::new(service(|svc| {
            println!("Layer called");
            WrappedService { inner: svc }
        }));
        let _svc = layer.call("foo");

        let cloned = layer.clone();
        std::thread::spawn(move || {
            cloned.call("foo");
            println!("Thread finished");
        }).join();

        println!("{:?}", layer);

        assert_eq!(
            "Function { f: rioc::function::tests::layer_fn_has_useful_debug_impl::{{closure}} }".to_string(),
            format!("{:?}", layer),
        );
    }

    #[test]
    fn service_fn_exa() {
        let f = Function::new (|x: i32| "hello");
        assert!(f.call(1) == "hello")
    }
}