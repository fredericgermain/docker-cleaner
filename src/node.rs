use std::cell::RefCell;
use std::rc::Rc;

pub trait Node {
    fn id(&self) -> String;
    fn deps(&self) -> &Vec<Rc<RefCell<dyn Node>>>;
    fn deps_mut(&mut self) -> &mut Vec<Rc<RefCell<dyn Node>>>;
    fn rdeps(&self) -> &Vec<Rc<RefCell<dyn Node>>>;
    fn rdeps_mut(&mut self) -> &mut Vec<Rc<RefCell<dyn Node>>>;
    fn delete(&self) -> anyhow::Result<()>;
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum NodeErr {
    Nope,
}

pub struct MissingNode {
    pub id: String,
    pub deps: Vec<Rc<RefCell<dyn Node>>>,
    pub rdeps: Vec<Rc<RefCell<dyn Node>>>,
}


impl MissingNode {
}

impl Node for MissingNode {
    
    fn id(&self) -> String {
        format!("MissingNode:{}", self.id)
    }

    fn deps(&self) -> &Vec<Rc<RefCell<dyn Node>>> {
        &self.deps
    }

    fn deps_mut(&mut self) -> &mut Vec<Rc<RefCell<dyn Node>>> {
        &mut self.deps
    }

    fn rdeps(&self) -> &Vec<Rc<RefCell<dyn Node>>> {
        &self.rdeps
    }

    fn rdeps_mut(&mut self) -> &mut Vec<Rc<RefCell<dyn Node>>> {
        &mut self.rdeps
    }

    fn delete(&self) -> anyhow::Result<()> {
        Ok(())
    }
}