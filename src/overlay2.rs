use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, Context};
use crate::node::{Node, MissingNode};

pub struct Overlay2Node {
    id: String,
  //  short_link: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    path: PathBuf,
}

impl Node for Overlay2Node {
    fn id(&self) -> String {
        self.id.clone()
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn delete(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).context("Failed to remove overlay2 directory")
    }
}

pub fn analyze_overlay2(base_path: &Path, graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>) -> Result<()> {
    let overlay2_path = base_path.join("overlay2");
    let mut layer_map_id_to_short_link = HashMap::new();
    let mut layer_map_short_link_to_id = HashMap::new();

    // Step 1: Build layer maps
    for entry in fs::read_dir(&overlay2_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let id = path.file_name().unwrap().to_string_lossy().into_owned();
            let link_path = path.join("link");
            if link_path.exists() {
                let overlay2_id = format!("Overlay2:{}", &id);
                let short_link = fs::read_to_string(link_path)?.trim().to_string();
                layer_map_id_to_short_link.insert(id.clone(), short_link.clone());
                layer_map_short_link_to_id.insert(short_link.clone(), id);

                let node = Rc::new(RefCell::new(Overlay2Node {
                    id: overlay2_id.clone(),
            //        short_link,
                    deps: Vec::new(),
                    rdeps: Vec::new(),
                    path,
                }));
                graph.insert(overlay2_id, node);
            }
        }
    }

    // Step 2: Build the graph
    for (id, _short_link) in &layer_map_id_to_short_link {
        let overlay2_id = format!("Overlay2:{}", id); 
        let node = match graph.get(&overlay2_id).map(Rc::clone) {
            Some(node) => node,
            None => continue,
        };

        let path = overlay2_path.join(&id);
        let lower_path = path.join("lower");

        if lower_path.exists() {
            let lower_content = fs::read_to_string(lower_path)?;
            for lower_short_link in lower_content.split(':') {
                let lower_short_id = lower_short_link.trim_start_matches("l/").to_string();
                if let Some(lower_id) = layer_map_short_link_to_id.get(&lower_short_id) {
                    let lower_node_id = format!("Overlay2:{}", lower_id);
                    if let Some(lower_node) = graph.get(&lower_node_id) {
                        (*node).borrow_mut().deps_mut().push(Rc::clone(lower_node));
                        (**lower_node).borrow_mut().rdeps_mut().push(Rc::clone(&node));
                    } else {
                        let mut rdeps = Vec::new();
                        rdeps.push(Rc::clone(&node));
                        let missing_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MissingNode {
                            id: lower_id.clone(),
                            deps: Vec::new(),
                            rdeps,
                        }));
                        (*node).borrow_mut().deps_mut().push(Rc::clone(&missing_node));
                        graph.insert(lower_node_id, missing_node);
                    }
                }
            }
        }
    }

    Ok(())
}