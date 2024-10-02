use std::collections::{HashMap, HashSet, VecDeque};

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

    for (key, node) in graph {
        let id = node.borrow().id();
        if &id != key { continue; }
        let node_type = id.split(':').next().unwrap_or("Unknown").to_string();
        classified.entry(node_type).or_insert_with(Vec::new).push(Rc::clone(node));
    }

    classified
}

pub fn remove_node_list(graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>, node_id: &str, recursive: bool) -> Result<Vec<Rc<RefCell<dyn Node>>>> {
    let mut result = Vec::new();
    if recursive {
        let mut visited = HashSet::new();
        let mut stack = VecDeque::new();
    
        stack.push_back(Rc::clone(&graph.get(node_id).unwrap()));
    
        while let Some(current) = stack.pop_front() {
            for dep in current.borrow().deps().iter().rev() {
                if dep.borrow().rdeps().len() <= 1 {
                    stack.push_back(dep.clone());
                }
            }
            if !visited.contains(&current.borrow().id()) {
                visited.insert(current.borrow().id());
                result.push(Rc::clone(&current));
            }
        }
    } else {
    
        let node = graph.get(node_id).unwrap();
        result.push(Rc::clone(node));
    }
    Ok(result)
}

pub fn remove_node(graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>, node: Rc<RefCell<dyn Node>>, recursive: bool) -> Result<()> {
    if recursive {
        // dfs, deletion on pre-order, to exit on 1st error, but only mess with a single branch in case of error
        let mut visited = HashSet::new();
        let mut stack = VecDeque::new();

        stack.push_back(node);
    
        while let Some(node) = stack.pop_back() {
            if visited.contains(&node.borrow().id()) {
                continue;
            }

            visited.insert(node.borrow().id());

            let result = node.borrow_mut().delete();
            match result {
                Ok(_) => {
                    graph.remove(&node.borrow().id());

                    // Push neighbors onto the stack in reverse order
                    // This ensures we visit them in the original order when popping
                    for dep in node.borrow().deps().iter().rev() {
                        let dep_id = node.borrow().id();

                        dep.borrow_mut().rdeps_mut().retain(|node| node.borrow().id() != dep_id);

                        if dep.borrow().rdeps().len() == 0 {
                            stack.push_back(Rc::clone(&dep));
                        }
                    }
                    node.borrow_mut().deps_mut().clear();                }
                Err(e) => {
                    eprintln!("error removing {}", &node.borrow().id());
                    return Err(e)
                }
            }
        }
    } else {
        match node.borrow().delete() {
            Ok(_) => {
                graph.remove(&node.borrow().id());
                for dep in node.borrow().deps() {
                    dep.borrow_mut().rdeps_mut().retain(|rdep| !Rc::ptr_eq(rdep, &node));
                }
                for rdep in node.borrow().rdeps() {
                    rdep.borrow_mut().deps_mut().retain(|dep| !Rc::ptr_eq(dep, &node));
                }
            }
            Err(e) => {
                eprintln!("error removing {}", &node.borrow().id());
                return Err(e)
            }
        };
    }

    Ok(())
}