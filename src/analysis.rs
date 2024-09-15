use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::Result;
use crate::node::Node;
use crate::overlay2::analyze_overlay2;
use crate::image::analyze_images;
use crate::container::analyze_containers;

pub fn build_graph(base_path: &Path) -> Result<HashMap<String, Rc<RefCell<dyn Node>>>> {
    let mut graph = HashMap::new();

    analyze_overlay2(base_path, &mut graph)?;
    analyze_images(base_path, &mut graph)?;
    analyze_containers(base_path, &mut graph)?;

    Ok(graph)
}

pub fn classify_layers(graph: &HashMap<String, Rc<RefCell<dyn Node>>>) -> HashMap<String, Vec<Rc<RefCell<dyn Node>>>> {
    let mut classified = HashMap::new();

    for node in graph.values() {
        let id = node.borrow().id();
        let node_type = id.split(':').next().unwrap_or("Unknown").to_string();
        classified.entry(node_type).or_insert_with(Vec::new).push(Rc::clone(node));
    }

    classified
}

pub fn remove_node(graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>, node_id: &str, recursive: bool) -> Result<()> {
    let mut to_remove = VecDeque::new();
    to_remove.push_back(node_id.to_string());

    while let Some(id) = to_remove.pop_front() {
        if let Some(node) = graph.remove(&id) {
            node.borrow().delete()?;

            if recursive {
                for dep in node.borrow().deps() {
                    let dep_id = dep.borrow().id();
                    dep.borrow_mut().inc_used_count(-1);
                    if dep.borrow().used_count() == 0 {
                        to_remove.push_back(dep_id);
                    }
                }
            }
        }
    }

    Ok(())
}