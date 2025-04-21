use rioc::{inject, injectable, provider};

#[injectable]
#[provider]
struct InitProvider;

#[injectable]
pub struct Controller {
    value: i32,
}

#[test]
fn test_basic() {
    #[injectable]
    #[provider]
    #[provide(i32, 123)]
    #[derive(Debug)]
    struct Provider {
    };

    let provider = InitProvider.provide::<Provider>();
    let controller = provider.provide::<Controller>();
    assert_eq!(controller.value, 123);
}

#[injectable]
pub struct Dep;

impl Dep {
    pub fn  welcome(self) {
        println!("welcome");
    }
}


#[injectable]
pub struct ControllerWithInject {
    dep: Dep,
    #[inject(200)]
    value: i32,
}
#[test]
fn test_basic_inject() {
    #[injectable]
    #[provider]
    #[provide(i32, 123)]
    #[derive(Debug)]
    struct Provider {
    };

    let provider = InitProvider.provide::<Provider>();
    let cc = provider.provide::<ControllerWithInject>();
    assert_eq!(cc.value, 200);
    cc.dep.welcome();
}