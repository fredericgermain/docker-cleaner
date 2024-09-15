use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct Layer {
    short_link: String,
    id: String,
    lower: RefCell<Vec<Rc<Layer>>>,
    used_count: usize,
}

pub fn read_repositories(base_path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let repo_path = base_path.join("image/overlay2/repositories.json");
    let content = fs::read_to_string(repo_path)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut repositories = HashMap::new();

    if let Value::Object(repos) = &json["Repositories"] {
        for (repo_name, images) in repos {
            if let Value::Object(image_tags) = images {
                for (tag, sha) in image_tags {
                    if let Value::String(sha_str) = sha {
                        let key = if tag.starts_with(&format!("{}@sha256:", repo_name)) {
                            tag.to_string()
                        } else {
                            format!("{}:{}", repo_name, tag)
                        };
                        repositories.insert(key, sha_str.trim_start_matches("sha256:").to_string());
                    }
                }
            }
        }
    }

    Ok(repositories)
}

pub fn read_image_contents(base_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content_path = base_path.join("image/overlay2/imagedb/content/sha256");
    let mut contents = Vec::new();

    for entry in fs::read_dir(content_path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            contents.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    Ok(contents)
}

pub fn analyze_overlay(
    base_path: &Path,
    repositories: &HashMap<String, String>,
    image_contents: &[String],
) -> Result<(HashMap<String, Rc<Layer>>, Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let overlay_path = base_path.join("overlay2");
    let mut layer_map = HashMap::new();
    let mut layer_map_id_to_short_link = HashMap::new();
    let mut layer_map_short_link_to_id = HashMap::new();

    // Step 3a: Build layer maps
    for entry in fs::read_dir(&overlay_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let id = path.file_name().unwrap().to_string_lossy().into_owned();
            let link_file = path.join("link");
            if link_file.exists() {
                let short_link = fs::read_to_string(link_file)?.trim().to_string();
                layer_map_id_to_short_link.insert(id.clone(), short_link.clone());
                layer_map_short_link_to_id.insert(short_link.clone(), id.clone());
                layer_map.insert(id.clone(), Rc::new(Layer {
                    short_link,
                    id,
                    lower: RefCell::new(Vec::new()),
                    used_count: 0,
                }));
            }
        }
    }

    // Step 3b: Build tree structure
    let mut queue = VecDeque::new();
    for (id, layer) in &layer_map {
        queue.push_back(id.clone());
    }

    while let Some(id) = queue.pop_front() {
        let layer = layer_map.get(&id).unwrap().clone();
        let lower_file = overlay_path.join(&id).join("lower");
        if lower_file.exists() {
            let lower_content = fs::read_to_string(lower_file)?;
            for lower_short_link in lower_content.split(':') {
                if let Some(lower_id) = layer_map_short_link_to_id.get(lower_short_link) {
                    if let Some(lower_layer) = layer_map.get(lower_id) {
                        layer.lower.borrow_mut().push(lower_layer.clone());
                        lower_layer.clone().used_count += 1;
                    }
                }
            }
        }
    }

    // Identify dangling images and orphan layers
    let mut dangling_images = Vec::new();
    let mut orphan_layers = Vec::new();

    for (id, layer) in &layer_map {
        if layer.used_count == 0 {
            if image_contents.contains(id) {
                dangling_images.push(id.clone());
            } else {
                orphan_layers.push(id.clone());
            }
        }
    }

    Ok((layer_map, dangling_images, orphan_layers))
}

pub fn display_results(
    layer_map: &HashMap<String, Rc<Layer>>,
    dangling_images: &[String],
    orphan_layers: &[String],
) {
    println!("Layer hierarchy:");
    for (id, layer) in layer_map {
        if layer.used_count == 0 {
            display_layer(layer, 0);
        }
    }

    println!("\nDangling images:");
    for id in dangling_images {
        println!("  {}", id);
    }

    println!("\nOrphan layers:");
    for id in orphan_layers {
        println!("  {}", id);
    }
}

fn display_layer(layer: &Rc<Layer>, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}{} ({:.32}) [used: {}]", indent, layer.short_link, layer.id, layer.used_count);
    for child in layer.lower.borrow().iter() {
        display_layer(child, depth + 1);
    }
}

pub fn propose_deletions(
    base_path: &Path,
    dangling_images: &[String],
    orphan_layers: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_delete = Vec::new();

    for id in dangling_images {
        let image_path = base_path.join(format!("image/overlay2/imagedb/content/sha256/{}", id));
        let overlay_path = base_path.join(format!("overlay2/{}", id));
        
        println!("Delete dangling image {}?", id);
        println!("  Image path: {}", image_path.display());
        println!("  Overlay path: {}", overlay_path.display());
        
        if confirm_deletion() {
            to_delete.push(image_path);
            to_delete.push(overlay_path);
        }
    }

    for id in orphan_layers {
        let overlay_path = base_path.join(format!("overlay2/{}", id));
        
        println!("Delete orphan layer {}?", id);
        println!("  Overlay path: {}", overlay_path.display());
        
        if confirm_deletion() {
            to_delete.push(overlay_path);
        }
    }

    for path in to_delete {
        println!("Deleting: {}", path.display());
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn confirm_deletion() -> bool {
    print!("Confirm deletion? (y/N): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_lowercase() == "y"
}
