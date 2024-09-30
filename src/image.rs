use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, Context};
use serde_json::Value;
use crate::node::{MissingNode, Node, StaticId};

const LAYERDB_PATH: &str ="image/overlay2/layerdb/sha256";
const IMAGEDB_PATH: &str ="image/overlay2/imagedb/content/sha256";
const METADATA_DIFFID_PATH: &str = "image/overlay2/distribution/v2metadata-by-diffid/sha256";
const DIGESTID_PATH: &str = "image/overlay2/distribution/diffid-by-digest/sha256";

pub struct ImageLayerNode {
    layer_id: String,
 //   layer_id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    base_path: PathBuf,
}

impl StaticId for ImageLayerNode {
    fn static_id(id: &str) -> String {
        format!("ImageLayer:{}", id)
    }
}

impl Node for ImageLayerNode {
    fn id(&self) -> String {
        Self::static_id(&self.layer_id)
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

    fn delete(&self) -> Result<()> {
        match fs::remove_dir_all(self.base_path.join(LAYERDB_PATH).join(&self.layer_id)).context("Failed to remove image layer directory") {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to dir file: {}", e),
        };
        Ok(())
    }
}


pub struct ImageContentNode {
    image_id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    base_path: PathBuf,
}

impl StaticId for ImageContentNode {
    fn static_id(id: &str) -> String {
        format!("ImageContent:{}", id)
    }
}

impl Node for ImageContentNode {
    fn id(&self) -> String {
        Self::static_id(&self.image_id)
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

    fn delete(&self) -> Result<()> {
        match fs::remove_dir_all(self.base_path.join(IMAGEDB_PATH).join(&self.image_id)).context("Failed to remove image content file") {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to dir file: {}", e),
        };
        Ok(())
    }
}

#[allow(dead_code)]
pub struct MetadataDiffIdNode {
    id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    digest: String,
    source_repository: Option<String>,
    base_path: PathBuf,
}

impl Node for MetadataDiffIdNode {
    fn id(&self) -> String {
        Self::static_id(&self.id)
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
    fn delete(&self) -> Result<()> {
        match fs::remove_file(self.base_path.join(METADATA_DIFFID_PATH).join(&self.id)) {
            Ok(()) => {},
            Err(e) => eprintln!("Failed to remove file: {}", e),
        }
        match fs::remove_file(self.base_path.join(DIGESTID_PATH).join(&self.digest)) {
            Ok(()) => {},
            Err(e) => eprintln!("Failed to remove file: {}", e),
        }
        Ok(())
    }
}

impl StaticId for MetadataDiffIdNode {
    fn static_id(id: &str) -> String {
        format!("MetadataDiffId:{}", id)
    }
}

#[allow(dead_code)]
pub struct LayerDiffIdNode {
    id: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
    base_path: PathBuf,
}

impl StaticId for LayerDiffIdNode {
    fn static_id(id: &str) -> String {
        format!("LayerDiffId:{}", id)
    }
}

impl Node for LayerDiffIdNode {
    fn id(&self) -> String {
        Self::static_id(&self.id)
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
    fn delete(&self) -> Result<()> {
        Ok(())
    }
}


pub struct ImageRepoNode {
    name_tag: String,
    deps: Vec<Rc<RefCell<dyn Node>>>,
    rdeps: Vec<Rc<RefCell<dyn Node>>>,
}

impl StaticId for ImageRepoNode {
    fn static_id(id: &str) -> String {
        format!("ImageRepo:{}", id)
    }
}

impl Node for ImageRepoNode {
    fn id(&self) -> String {
        Self::static_id(&self.name_tag)
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

    fn delete(&self) -> Result<()> {
        Ok(()) // Repositories are not deleted directly
    }
}

pub fn analyze_images(base_path: &Path, graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>) -> Result<()> {

    // Analyse diff ID
    let diffid_fullpath = base_path.join(METADATA_DIFFID_PATH);
 // also would need to check pending file in DIGESTID_PATH
    for entry in fs::read_dir(&diffid_fullpath)? {
        let entry = entry?;
        let diff_id = entry.file_name().to_string_lossy().into_owned();
        let file_path = entry.path();
        let content = fs::read_to_string(&file_path)?;
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
            let diff_id_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(MetadataDiffIdNode {
                id: diff_id.clone(),
       //        layer_id: layer_id.clone(),
                deps: Vec::new(),
                rdeps: Vec::new(),
                digest: digest.to_string(),
                source_repository: Some(json["SourceRepository"].as_str().unwrap_or("").to_string()),
                base_path: base_path.to_path_buf(),
            }));
            let diff_id_node_id = diff_id_node.borrow().id();
            graph.insert(diff_id_node_id, diff_id_node);
        }
    }


    // Analyze layer diff IDs
    let layerdb_path = base_path.join(LAYERDB_PATH);
    for entry in fs::read_dir(&layerdb_path)? {
        let entry = entry?;
        let layer_id = entry.file_name().to_string_lossy().into_owned();

        let image_layer_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(ImageLayerNode {
            layer_id: layer_id.clone(),
   //        layer_id: layer_id.clone(),
            deps: Vec::new(),
            rdeps: Vec::new(),
            base_path: base_path.to_path_buf(),
        }));

        let cache_id_path = entry.path().join("cache-id");
        let cache_overlay_id = fs::read_to_string(cache_id_path)?.trim().to_string();
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
        let diff_id_id = diff_content.trim_start_matches("sha256:");
        /*
        let diff_id_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(LayerDiffIdNode {
            id: diff_id_id.to_string(),
   //        layer_id: layer_id.clone(),
            deps: Vec::new(),
            rdeps: Vec::new(),
            base_path: base_path.to_path_buf(),
        }));

        let diff_id_node_id = diff_id_node.borrow().id();
        diff_id_node.borrow_mut().deps_mut().push(Rc::clone(&image_layer_node));
        image_layer_node.borrow_mut().rdeps_mut().push(Rc::clone(&diff_id_node));

        graph.insert(diff_id_node_id, diff_id_node);
        */
        graph.insert(LayerDiffIdNode::static_id(diff_id_id), Rc::clone(&image_layer_node));

        let metadata_diff_id_node_id = format!("MetadataDiffId:{}", diff_id_id);
        match graph.get(&metadata_diff_id_node_id) {
            Some(metadata_diff_id_node) => {
                image_layer_node.borrow_mut().deps_mut().push(Rc::clone(&metadata_diff_id_node));
                metadata_diff_id_node.borrow_mut().rdeps_mut().push(Rc::clone(&image_layer_node));
//                println!("found metadata_diff_id for ImageLayerNode {} {} in {}", &layer_id, &metadata_diff_id_node_id, METADATA_DIFFID_PATH);
            }
            None => {
//                println!("no metadata_diff_id for ImageLayerNode {} {}", &layer_id, &metadata_diff_id_node_id);
            }
        }
        graph.insert(ImageLayerNode::static_id(&layer_id), image_layer_node);
    }
    for entry in fs::read_dir(&layerdb_path)? {
        let entry = entry?;
        let layer_id = entry.file_name().to_string_lossy().into_owned();

        let image_layer_node = graph.get(&ImageLayerNode::static_id(&layer_id)).unwrap();

        let layer_parent_id_path = entry.path().join("parent");
        if let Ok(layer_parent_id) = fs::read_to_string(layer_parent_id_path) {
            let layer_parent_id = layer_parent_id.trim().to_string();
            let layer_parent_id = layer_parent_id.trim_start_matches("sha256:");

            let layer_parent_node = graph.get(&ImageLayerNode::static_id(&layer_parent_id)).unwrap();

            layer_parent_node.borrow_mut().deps_mut().push(Rc::clone(&image_layer_node));
            image_layer_node.borrow_mut().rdeps_mut().push(Rc::clone(&layer_parent_node));
        }
    }

    // Analyze image content
    let imagedb_path = base_path.join(IMAGEDB_PATH);
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
                        base_path: base_path.to_path_buf(),
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