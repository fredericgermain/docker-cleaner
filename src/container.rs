use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, Context};
use serde_json::Value;
use crate::node::{Node, MissingNode};

pub struct ContainerNode {
    container_id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    used_count: usize,
    path: PathBuf,
}

impl Node for ContainerNode {
    fn id(&self) -> String {
        format!("Container:{}", self.container_id)
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn delete(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).context("Failed to remove container directory")
    }
}


pub struct MountNode {
    mount_id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    used_count: usize,
    path: PathBuf,
}

impl Node for MountNode {
    fn id(&self) -> String {
        format!("Mount:{}", self.mount_id)
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn delete(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).context("Failed to remove mount directory")
    }
}

pub fn analyze_containers(base_path: &Path, graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>) -> Result<()> {
    let containers_path = base_path.join("containers");
    let mounts_path = base_path.join("image/overlay2/layerdb/mounts");

    for entry in fs::read_dir(&mounts_path)? {
        let entry = entry?;
        let mount_id = entry.file_name().to_string_lossy().into_owned();
        let mount_path = mounts_path.join(&mount_id);

        let mount_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MountNode {
            mount_id: mount_id.clone(),
            deps: Vec::new(),
            used_count: 1,
            path: mount_path.clone(),
        }));

        // Add dependencies on mount layers
        for layer_file in ["init-id", "mount-id"] {
            let layer_path = mount_path.join(layer_file);
            if layer_path.exists() {
                let layer_id = fs::read_to_string(layer_path)?.trim().to_string();
                let overlay_id = format!("Overlay2:{}", layer_id);
                if let Some(overlay_node) = graph.get(&overlay_id) {
                    mount_node.borrow_mut().deps_mut().push(Rc::clone(overlay_node));
                    overlay_node.borrow_mut().inc_used_count(1);
                } else {
                    let missing_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MissingNode {
                        id: overlay_id.clone(),
                        deps: Vec::new(),
                        used_count: 1,
                    }));
                    mount_node.borrow_mut().deps_mut().push(Rc::clone(&missing_node));
                    graph.insert(overlay_id, missing_node);
                }
            }
        }
        let mount_node_id = mount_node.borrow().id();
        graph.insert(mount_node_id, mount_node);
    }
    for entry in fs::read_dir(&containers_path)? {
        let entry = entry?;
        let container_id = entry.file_name().to_string_lossy().into_owned();
        let config_path = entry.path().join("config.v2.json");
        let config_content = fs::read_to_string(&config_path);
        if let Err(error) = config_content {
            println!("no config.v2.json for {} {}", config_path.to_str().unwrap_or_default(), error);
            continue;
        }
        let config_content = config_content.unwrap();
        let config: Result<Value, serde_json::Error> = serde_json::from_str(&config_content);
        
        match config {
            Ok(config) => {
                let image_id = config["Image"].as_str().unwrap_or("").trim_start_matches("sha256:");
                let mut deps = Vec::new();
    
                // Add dependency on the image content
                let image_content_id = format!("ImageContent:{}", image_id);
                if let Some(image_node) = graph.get(&image_content_id) {
                    deps.push(Rc::clone(image_node));
                    image_node.borrow_mut().inc_used_count(1);
                } else {
                    let missing_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MissingNode {
                        id: image_content_id.clone(),
                        deps: Vec::new(),
                        used_count: 1,
                    }));
                    deps.push(Rc::clone(&missing_node));
                    graph.insert(image_content_id, missing_node);
                }
    
                let mount_id = format!("Mount:{}", container_id);
                match graph.get(&mount_id) {
                    Some(node) => {
                        deps.push(node.clone());
                        node.borrow_mut().inc_used_count(1);
                    }
                    None => {
                        let missing_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MissingNode {
                            id: mount_id.clone(),
                            deps: Vec::new(),
                            used_count: 1,
                        }));
                        deps.push(Rc::clone(&missing_node));
                        let missing_node_id = missing_node.borrow().id();
                        graph.insert(missing_node_id, missing_node);
    
                    }
                }
    
                // Add dependencies on mount layers
                let mount_path = mounts_path.join(&container_id);
                for layer_file in ["init-id", "mount-id"] {
                    let layer_path = mount_path.join(layer_file);
                    if layer_path.exists() {
                        let layer_id = fs::read_to_string(layer_path)?.trim().to_string();
                        let overlay_id = format!("Overlay2:{}", layer_id);
                        if let Some(overlay_node) = graph.get(&overlay_id) {
                            deps.push(Rc::clone(overlay_node));
                            overlay_node.borrow_mut().inc_used_count(1);
                        } else {
                        }
                    }
                }
    
                let container_node = Rc::new(RefCell::new(ContainerNode {
                    container_id: container_id.clone(),
                    deps,
                    used_count: 0,
                    path: entry.path(),
                }));
                graph.insert(format!("Container:{}", container_id), container_node);    
            },
            Err(error) => { println!("could not parse {}", config_path.to_str().unwrap_or_default())}
        }
    }

    Ok(())
}