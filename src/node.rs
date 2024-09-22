use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Node {
    fn id(&self) -> String;
    fn deps(&self) -> &Vec<Rc<RefCell<dyn Node>>>;
    fn deps_mut(&mut self) -> &mut Vec<Rc<RefCell<dyn Node>>>;
    fn used_count(&self) -> usize;
    fn inc_used_count(&mut self, count: isize);
    fn as_any(&self) -> &dyn Any;
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
    pub used_count: usize,
}


impl MissingNode {
    pub fn new(id: String, used_count: usize) -> Self {
        MissingNode {
            id,
            deps: Vec::new(),
            used_count,
        }
    }
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

    fn used_count(&self) -> usize {
        self.used_count
    }

    fn inc_used_count(&mut self, count: isize) {
        self.used_count = (self.used_count as isize + count) as usize;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn delete(&self) -> anyhow::Result<()> {
        Ok(())
    }
}