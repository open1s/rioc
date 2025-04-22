use std::{cell::RefCell, collections::HashMap};
use std::clone;
use std::collections::VecDeque;
use std::error::Error;
use std::rc::Rc;

use crate::function::{service, Function, Service};

#[derive(Debug, Clone)]
pub struct ChainContext {
    pub data: HashMap<String,String>,
}

#[derive(Debug, Clone)]
pub struct PayLoad {
    pub data: Option<String>,
    pub ctx: Option<ChainContext>,
}

#[derive(Clone, Debug)]
pub enum Direction{
   Inbound,
   Outbound,
}

#[derive(Debug, Clone)]
pub struct LayerResult {
    pub direction: Direction,
    pub data: Option<PayLoad>,
}


pub struct ProtocolAware{
    func: Box<dyn Fn(PayLoad) -> LayerResult>,
}

impl Service<PayLoad,LayerResult> for ProtocolAware {
    fn call(&self, input: PayLoad) -> LayerResult {
        (self.func)(input)
    }
}

pub fn protocol_handler(f: impl Fn(PayLoad) -> LayerResult + 'static) -> ProtocolAware {
   ProtocolAware { func: Box::new(f)}
}

pub type SharedLayer = Rc<RefCell<Layer>>;

#[derive(Clone)]
pub struct Layer {
    pub handle_inbound: Rc<Box<ProtocolAware>>,
    pub handle_outbound: Rc<Box<ProtocolAware>>,
    pub lo_layer: Option<SharedLayer>,
    pub up_layer: Option<SharedLayer>,
}

impl Layer {
    pub fn new(
        handle_inbound: Rc<Box<ProtocolAware>>,
        handle_outbound: Rc<Box<ProtocolAware>>,
    ) -> Self {
        Self {
            handle_inbound,
            handle_outbound,
            lo_layer: None,
            up_layer: None,
        }
    }

    pub fn handle_inbound(&self, req: PayLoad) -> Result<(), String> {
        // 先执行 call，拿到结果，避免嵌套 borrow
        let result = self.handle_inbound.call(req);

        let (direction, data) = (result.direction, result.data);

        let upstream = self.up_layer.clone();
        let downstream = self.lo_layer.clone();

        match direction {
            Direction::Inbound => {
                if let Some(upstream) = upstream {
                    if let Some(data) = data {
                        upstream.borrow().handle_inbound(data)?;
                    }
                }
            }
            Direction::Outbound => {
                if let Some(downstream) = downstream {
                    if let Some(data) = data {
                        downstream.borrow().handle_outbound(data)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn handle_outbound(&self, req: PayLoad) ->  Result<(), String> {
        // 先执行 call，拿到结果，避免嵌套 borrow
        let result = self.handle_outbound.call(req);

        let (direction, data) = (result.direction, result.data);

        let upstream = self.up_layer.clone();
        let downstream = self.lo_layer.clone();

        match direction {
            Direction::Inbound => {
                if let Some(upstream) = upstream {
                    if let Some(data) = data {
                        upstream.borrow().handle_inbound(data)?;
                    }
                }
            }
            Direction::Outbound => {
                if let Some(downstream) = downstream {
                    if let Some(data) = data {
                        downstream.borrow().handle_outbound(data)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct LayerBuilder {
    hanlde_inbound: Option<Rc<Box<ProtocolAware>>>,
    handle_outbound: Option<Rc<Box<ProtocolAware>>>,
}

impl LayerBuilder {
    pub fn new() -> Self {
        Self {
            hanlde_inbound: None,
            handle_outbound: None,
        }
    }

    pub fn with_inbound_fn(
        mut self,
        handle: impl Fn(PayLoad) -> LayerResult + 'static,
    ) -> Self {
        let handle = ProtocolAware { func: Box::new(handle) };
        self.hanlde_inbound = Some(Rc::new(Box::new(handle)));
        self
    }

    pub fn with_outbound_fn(
        mut self,
        handle: impl Fn(PayLoad) -> LayerResult + 'static,
    ) -> Self {
        let handle = ProtocolAware { func: Box::new(handle) };
        self.handle_outbound = Some(Rc::new(Box::new(handle)));
        self
    }

    pub fn build(self) -> Result<Rc<RefCell<Layer>>, String> {
        let inbound = self.hanlde_inbound.ok_or("inbound handler not set")?;
        let outbound = self.handle_outbound.ok_or("outbound handler not set")?;
        Ok(Rc::new(RefCell::new(Layer {
            handle_inbound: inbound,
            handle_outbound: outbound,
            up_layer: None,
            lo_layer: None,
        })))
    }
}

pub struct LayerChain {
    head: Option<SharedLayer>,
    tail: Option<SharedLayer>,
}

impl LayerChain {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn add_layer(&mut self, layer: SharedLayer) {
        match self.tail.take() {
            Some(tail) => {
                // tail -> new layer
                tail.borrow_mut().up_layer = Some(layer.clone());
                // new layer -> tail
                layer.borrow_mut().lo_layer = Some(tail.clone());
                self.tail = Some(layer);
            }
            None => {
                layer.borrow_mut().lo_layer = None;
                layer.borrow_mut().up_layer = None;
                self.head = Some(layer.clone());
                self.tail = Some(layer);
            }
        }
    }

    pub fn head(&self) -> Option<SharedLayer> {
        self.head.clone()
    }

    pub fn tail(&self) -> Option<SharedLayer> {
        self.tail.clone()
    }

    pub fn handle_inbound(&self, req: PayLoad) -> Result<(), String>  {
        if self.head.is_none() {
            return Err("No layers in the chain".into());
        }

        let head = self.head.clone().unwrap();
        let _ = head.borrow().handle_inbound(req.clone());
        Ok(())
    }

    pub fn handle_outbound(&self, req: PayLoad) -> Result<(), String> {
        if self.tail.is_none() {
            return Err("No layers in the chain".into());
        }
        let tail = self.tail.clone().unwrap();
        let _ = tail.borrow().handle_outbound(req.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_chain() {
        let chain = LayerChain::new();
        let req = PayLoad {
            data: Some("test".to_string()),
            ctx: None,
        };
        
        assert!(chain.handle_inbound(req.clone()).is_err());
        assert!(chain.handle_outbound(req).is_err());
    }

    #[test]
    fn test_single_layer_chain() {
        let layer = LayerBuilder::new()
            .with_inbound_fn(|req| {
                println!("layer inbound: {:?}", req);
                LayerResult {
                    direction: Direction::Inbound,
                    data: Some(PayLoad {
                        data: req.data,
                        ctx: req.ctx,
                    }),
                }
            })
            .with_outbound_fn(|req| {
                println!("layer outbound: {:?}", req);
                LayerResult {
                    direction: Direction::Outbound,
                    data: Some(PayLoad {
                        data: req.data,
                        ctx: req.ctx,
                    }),
                }
            })
            .build().unwrap();

        let mut chain = LayerChain::new();
        chain.add_layer(layer);
        
        let req = PayLoad {
            data: Some("test".to_string()),
            ctx: None,
        };
        
        assert!(chain.handle_inbound(req.clone()).is_ok());
        assert!(chain.handle_outbound(req).is_ok());
    }

    #[test]
    fn test_layer_builder() {
       let layer0 = LayerBuilder::new().with_inbound_fn(|req|{
           println!("layer0 inbound: {:?}", req);
           LayerResult {
              direction: Direction::Inbound,
              data: Some(PayLoad {
                  data: req.data,
                  ctx:None,
              }),
           }
       })
       .with_outbound_fn(|req|{
           println!("layer0 outbound: {:?}", req);
           LayerResult {
              direction: Direction::Outbound,
              data: Some(PayLoad {
                  data: req.data,
                  ctx: None,
              }),
           }
       })
       .build().unwrap();

       let layer1 = LayerBuilder::new().with_inbound_fn(|req|{
           println!("layer1 inbound: {:?}", req);
           LayerResult {
              direction: Direction::Inbound,
              data: Some(PayLoad {
                  data: req.data,
                  ctx: None,
              }),
           }
       })
      .with_outbound_fn(|req|{
         println!("layer1 outbound: {:?}", req);
         LayerResult { 
            direction: Direction::Outbound, 
            data: Some(PayLoad {
                data: req.data,
                ctx: None,
            })
         }
      })
      .build().unwrap();

       let mut chain = LayerChain::new();
       chain.add_layer(layer0);
       chain.add_layer(layer1);


       let req = PayLoad {       
          data: Some("hello".to_string()),
          ctx: None
        };
          
       chain.handle_inbound(req).unwrap();
       let req = PayLoad {       
            data: Some("hello".to_string()),
            ctx: None
        };
       chain.handle_outbound(req).unwrap();
    }
}