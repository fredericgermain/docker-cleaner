use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, Context};
use serde_json::Value;
use crate::node::{Node, MissingNode};

pub struct ImageLayerNode {
    image_id: String,
 //   layer_id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    path: PathBuf,
}

impl Node for ImageLayerNode {
    fn id(&self) -> String {
        format!("ImageLayer:{}", self.image_id)
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
        fs::remove_dir_all(&self.path).context("Failed to remove image layer directory")
    }
}

pub struct ImageContentNode {
    image_id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    path: PathBuf,
}

impl Node for ImageContentNode {
    fn id(&self) -> String {
        format!("ImageContent:{}", self.image_id)
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
        fs::remove_file(&self.path).context("Failed to remove image content file")
    }
}

#[allow(dead_code)]
pub struct DiffIdNode {
    id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    digest: String,
    source_repository: Option<String>,
}

impl Node for DiffIdNode {
    fn id(&self) -> String {
        format!("DiffId:{}", self.id)
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
        Ok(()) // Repositories are not deleted directly
    }
}

pub struct ImageRepoNode {
    name_tag: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
}

impl Node for ImageRepoNode {
    fn id(&self) -> String {
        format!("ImageRepo:{}", self.name_tag)
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
        Ok(()) // Repositories are not deleted directly
    }
}

pub fn analyze_images(base_path: &Path, graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>) -> Result<()> {
    let layerdb_path = base_path.join("image/overlay2/layerdb/sha256");
    let imagedb_path = base_path.join("image/overlay2/imagedb/content/sha256");

    // Analyse diff ID
    let diffid_path = base_path.join("image/overlay2/distribution/v2metadata-by-diffid/sha256");
 // also /image/overlay2/distribution/diffid-by-digest/sha256/$MS

    for entry in fs::read_dir(&diffid_path)? {
        let entry = entry?;
        let diff_id = entry.file_name().to_string_lossy().into_owned();
        let content = fs::read_to_string(entry.path())?;
        let json: Value = serde_json::from_str(&content)?;

        let diff_id_array = json.as_array().unwrap();
        /* there is often more than one entry in that file.
            it seems it's always the same digest in all those entreis
            there is a HMAC which is different though
        */
        /* if diff_id_array.len() > 1 {
            println!("more that one entry in {}", entry.path().to_str().unwrap());
        } */
        if let Some(sha_digest) = diff_id_array.get(0).unwrap().get("Digest") {
            let digest = sha_digest.as_str().unwrap_or_default().trim_start_matches("sha256:");
            let image_layer_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(DiffIdNode {
                id: diff_id.clone(),
       //        layer_id: layer_id.clone(),
                deps: Vec::new(),
                rdeps: Vec::new(),
                digest: digest.to_string(),
                source_repository: Some(json["SourceRepository"].as_str().unwrap_or("").to_string()),
            }));
            let overlay2_id = format!("DiffId:{}", diff_id);
            graph.insert(overlay2_id, image_layer_node);
        }
    }


    // Analyze layer diff IDs
    for entry in fs::read_dir(&layerdb_path)? {
        let entry = entry?;
        let layer_id = entry.file_name().to_string_lossy().into_owned();
        let cache_id_path = entry.path().join("cache-id");
       
        let cache_overlay_id = fs::read_to_string(cache_id_path)?.trim().to_string();
        let image_layer_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(ImageLayerNode {
            image_id: layer_id.clone(),
   //        layer_id: layer_id.clone(),
            deps: Vec::new(),
            rdeps: Vec::new(),
            path: entry.path(),
        }));
        let overlay2_id = format!("Overlay2:{}", cache_overlay_id);
        match graph.get(&overlay2_id) {
            Some(node) => {
                image_layer_node.borrow_mut().deps_mut().push(Rc::clone(node));
                node.borrow_mut().rdeps_mut().push(Rc::clone(&image_layer_node));
            }
            None => {
                let mut rdeps = Vec::new();
                rdeps.push(Rc::clone(&image_layer_node));
                let missing_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MissingNode {
                    id: overlay2_id,
                    deps: Vec::new(),
                    rdeps,
                }));
                image_layer_node.borrow_mut().deps_mut().push(Rc::clone(&missing_node));
                let id_missing = missing_node.borrow().id();
                graph.insert(id_missing, missing_node);
            }
        }
        
        let diff_path = entry.path().join("diff");
        let diff_content = fs::read_to_string(diff_path)?;
        let digest = diff_content.trim_start_matches("sha256:");
        let diffid_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(DiffIdNode {
            id: digest.to_string(),
   //        layer_id: layer_id.clone(),
            deps: Vec::new(),
            rdeps: Vec::new(),
            digest: digest.to_string(),
            source_repository: None,
        }));
        let overlay2_id = format!("DiffId:{}", digest);

        diffid_node.borrow_mut().deps_mut().push(Rc::clone(&image_layer_node));
        image_layer_node.borrow_mut().rdeps_mut().push(Rc::clone(&diffid_node));

        graph.insert(overlay2_id, diffid_node);
        graph.insert(format!("ImageLayer:{}", layer_id), image_layer_node);
    }

    // Analyze image content
    for entry in fs::read_dir(&imagedb_path)? {
        let entry = entry?;
        let image_id = entry.file_name().to_string_lossy().into_owned();
        let content = fs::read_to_string(entry.path())?;
        let json: Value = serde_json::from_str(&content)?;

        if let Some(rootfs) = json.get("rootfs") {
            if let Some(diff_ids) = rootfs.get("diff_ids") {
                if let Some(diff_ids) = diff_ids.as_array() {
                    let node = Rc::new(RefCell::new(ImageContentNode {
                        image_id: image_id.clone(),
                        deps: Vec::new(),
                        rdeps: Vec::new(),
                        path: entry.path(),
                    }));
                    for diff_id in diff_ids {
                        let layer_diff_id = diff_id.as_str().unwrap_or("").trim_start_matches("sha256:");
                        let layer_node_id = format!("DiffId:{}", layer_diff_id);
                        if let Some(layer_node) = graph.get(&layer_node_id) {
                            node.borrow_mut().deps.push(Rc::clone(layer_node));
                            layer_node.borrow_mut().rdeps_mut().push(Rc::clone(&node) as Rc<RefCell<dyn Node>>);
                        } else {
                            let mut rdeps: Vec<Rc<RefCell<dyn Node>>> = Vec::new();
                            rdeps.push(Rc::clone(&node) as Rc<RefCell<dyn Node>>);
                            let missing_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MissingNode {
                                id: layer_node_id.clone(),
                                deps: Vec::new(),
                                rdeps,
                            }));
                            node.borrow_mut().deps.push(Rc::clone(&missing_node));
                            graph.insert(layer_node_id, missing_node);
                        }
                    }
                    graph.insert(format!("ImageContent:{}", image_id), node);
                }
            }
        }
    }

    // Analyze repositories
    let repositories = read_repositories(base_path)?;
    for (name_tag, image_id) in repositories {
        let content_node_id = format!("ImageContent:{}", image_id);
        if let Some(content_node) = graph.get(&content_node_id) {
            let node = Rc::new(RefCell::new(ImageRepoNode {
                name_tag: name_tag.clone(),
                deps: vec![Rc::clone(content_node)],
                rdeps: Vec::new(),
            }));
            content_node.borrow_mut().rdeps_mut().push(Rc::clone(&node) as Rc<RefCell<dyn Node>>);
            graph.insert(format!("ImageRepo:{}", name_tag), node);
        }
    }

    Ok(())
}

fn read_repositories(base_path: &Path) -> Result<HashMap<String, String>> {
    let repo_file = base_path.join("image/overlay2/repositories.json");
    let content = fs::read_to_string(repo_file)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut repositories = HashMap::new();
    if let Some(repos) = json.get("Repositories") {
        if let Some(repos) = repos.as_object() {
            for (repo, tags) in repos {
                if let Some(tags) = tags.as_object() {
                    for (tag, digest) in tags {
                        let name_tag = format!("{}:{}", repo, tag);
                        let image_id = digest.as_str().unwrap_or("").trim_start_matches("sha256:");
                        repositories.insert(name_tag, image_id.to_string());
                    }
                }
            }
        }
    }

    Ok(repositories)
}