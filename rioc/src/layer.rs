use std::sync::Weak;
use std::{cell::RefCell, collections::HashMap};
use std::{any, clone};
use std::collections::VecDeque;
use std::error::Error;
use std::sync::Arc;

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
    func: Box<dyn Fn(Option<PayLoad>) -> Result<LayerResult, String>>,
}

impl Service<Option<PayLoad>,Result<LayerResult, String>> for ProtocolAware {
    fn call(&self, input: Option<PayLoad>) -> Result<LayerResult, String> {
        (self.func)(input)
    }
}

pub fn protocol_handler(f: impl Fn(Option<PayLoad>) -> Result<LayerResult, String> + 'static) -> ProtocolAware {
   ProtocolAware { func: Box::new(f)}
}

pub type SharedLayer = Arc<RefCell<Layer>>;
pub type WeakLayer = Weak<RefCell<Layer>>;

#[derive(Clone)]
pub struct Layer {
    pub handle_inbound: Arc<Box<ProtocolAware>>,
    pub handle_outbound: Arc<Box<ProtocolAware>>,
    pub lo_layer: Option<SharedLayer>,
    pub up_layer: Option<WeakLayer>,
}

impl Layer {
    pub fn new(
        handle_inbound: Arc<Box<ProtocolAware>>,
        handle_outbound: Arc<Box<ProtocolAware>>,
    ) -> Self {
        Self {
            handle_inbound,
            handle_outbound,
            lo_layer: None,
            up_layer: None,
        }
    }

    pub fn handle_inbound(&self, req: Option<PayLoad>) -> Result<LayerResult, String> {
        // 先执行 call，拿到结果，避免嵌套 borrow
        let result = self.handle_inbound.call(req);
        if result.is_err() {
            return Err("failed to handle inbound request".into());
        }
        let result = result.unwrap();
        let mut cloned_result = result.clone();

        let (direction, data) = (result.direction, result.data);

        let upstream = self.up_layer.clone();
        let downstream = self.lo_layer.clone();

        match direction {
            Direction::Inbound => {
                if let Some(upstream) = upstream {
                    if let Some(upstream) = upstream.upgrade(){
                        cloned_result = upstream.borrow().handle_inbound(data)?;
                    }else{
                        return Err("failed to handle inbound request".into());
                    }
                }
            }
            Direction::Outbound => {
                if let Some(downstream) = downstream {
                    cloned_result = downstream.borrow().handle_outbound(data)?;
                }
            }
        }

        Ok(cloned_result)
    }

    pub fn handle_outbound(&self, req: Option<PayLoad>) ->  Result<LayerResult, String> {
        // 先执行 call，拿到结果，避免嵌套 borrow
        let result: Result<LayerResult, String> = self.handle_outbound.call(req);
        if result.is_err() {
            return Err("failed to handle outbound request".into());
        }
        let result = result.unwrap();
        let mut cloned_result = result.clone();

        let (direction, data) = (result.direction, result.data);

        let upstream = self.up_layer.clone();
        let downstream = self.lo_layer.clone();

        match direction {
            Direction::Inbound => {
                if let Some(upstream) = upstream {
                    if let Some(upstream) = upstream.upgrade(){
                        cloned_result = upstream.borrow().handle_inbound(data)?;
                    }else {
                        return Err("failed to handle inbound request".into());
                    }               
                }
            }
            Direction::Outbound => {
                if let Some(downstream) = downstream {
                    cloned_result = downstream.borrow().handle_outbound(data)?;
                }
            }
        }

        Ok(cloned_result)
    }
}

pub struct LayerBuilder {
    hanlde_inbound: Option<Arc<Box<ProtocolAware>>>,
    handle_outbound: Option<Arc<Box<ProtocolAware>>>,
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
        handle: impl Fn(Option<PayLoad>) -> Result<LayerResult,String> + 'static,
    ) -> Self {
        let handle = ProtocolAware { func: Box::new(handle) };
        self.hanlde_inbound = Some(Arc::new(Box::new(handle)));
        self
    }

    pub fn with_outbound_fn(
        mut self,
        handle: impl Fn(Option<PayLoad>) -> Result<LayerResult,String> + 'static,
    ) -> Self {
        let handle = ProtocolAware { func: Box::new(handle) };
        self.handle_outbound = Some(Arc::new(Box::new(handle)));
        self
    }

    pub fn build(self) -> Result<Arc<RefCell<Layer>>, String> {
        let inbound = self.hanlde_inbound.ok_or("inbound handler not set")?;
        let outbound = self.handle_outbound.ok_or("outbound handler not set")?;
        Ok(Arc::new(RefCell::new(Layer {
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
                // tail.borrow_mut().up_layer = Some(layer.clone());
                tail.borrow_mut().up_layer = Some(Arc::downgrade(&layer));
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

    pub fn handle_inbound(&self, req: Option<PayLoad>) -> Result<LayerResult, String>  {
        if self.head.is_none() {
            return Err("No layers in the chain".into());
        }

        let head = self.head.clone().unwrap();
        let result = head.borrow().handle_inbound(req);
        result
    }

    pub fn handle_outbound(&self, req: Option<PayLoad>) -> Result<LayerResult, String> {
        if self.tail.is_none() {
            return Err("No layers in the chain".into());
        }
        let tail = self.tail.clone().unwrap();
        let result = tail.borrow().handle_outbound(req);
        result
    }
}

impl Drop for LayerChain {
    fn drop(&mut self) {
        self.head = None;
        self.tail = None;
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
        
        assert!(chain.handle_inbound(Some(req.clone())).is_err());
        assert!(chain.handle_outbound(Some(req)).is_err());
    }

    #[test]
    fn test_single_layer_chain() {
        let layer = LayerBuilder::new()
            .with_inbound_fn(|req| {
                println!("layer inbound: {:?}", req);
                let req = req.unwrap();
                Ok(LayerResult {
                    direction: Direction::Inbound,
                    data: Some(PayLoad {
                        data: req.data,
                        ctx:  req.ctx,
                    }),
                })
            })
            .with_outbound_fn(|req| {
                println!("layer outbound: {:?}", req);
                let req = req.unwrap();
                Ok(LayerResult {
                    direction: Direction::Outbound,
                    data: Some(PayLoad {
                        data: req.data,
                        ctx:  req.ctx,
                    }),
                })
            })
            .build().unwrap();

        let mut chain = LayerChain::new();
        chain.add_layer(layer);
        
        let req = PayLoad {
            data: Some("test".to_string()),
            ctx: None,
        };
        
        assert!(chain.handle_inbound(Some(req.clone())).is_ok());
        assert!(chain.handle_outbound(Some(req)).is_ok());
    }

    #[test]
    fn test_layer_builder() {
       let layer0 = LayerBuilder::new().with_inbound_fn(|req|{
           println!("layer0 inbound: {:?}", req);
           let req = req.unwrap();
           Ok(LayerResult {
              direction: Direction::Inbound,
              data: Some(PayLoad {
                  data: req.data,
                  ctx:None,
              }),
           })
       })
       .with_outbound_fn(|req|{
           println!("layer0 outbound: {:?}", req);
           let req = req.unwrap();
           Ok(LayerResult {
              direction: Direction::Outbound,
              data: Some(PayLoad {
                  data: req.data,
                  ctx: None,
              }),
           })
       })
       .build().unwrap();

       let layer1 = LayerBuilder::new().with_inbound_fn(|req|{
           println!("layer1 inbound: {:?}", req);
           let req = req.unwrap();
           Ok(LayerResult {
              direction: Direction::Inbound,
              data: Some(PayLoad {
                  data: req.data,
                  ctx: None,
              }),
           })
       })
      .with_outbound_fn(|req|{
         println!("layer1 outbound: {:?}", req);
         let req = req.unwrap();
         Ok(LayerResult { 
            direction: Direction::Outbound, 
            data: Some(PayLoad {
                data: req.data,
                ctx: None,
            })
         })
      })
      .build().unwrap();

       let mut chain = LayerChain::new();
       chain.add_layer(layer0);
       chain.add_layer(layer1);


       let req = PayLoad {       
          data: Some("hello".to_string()),
          ctx: None
        };
          
       chain.handle_inbound(Some(req)).unwrap();
       let req = PayLoad {       
            data: Some("hello".to_string()),
            ctx: None
        };
       chain.handle_outbound(Some(req)).unwrap();
    }
}