use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Layer {
    short_link: String,
    id: String,
    lower: Vec<String>, // Store layer IDs instead of Rc<RefCell<Layer>>
    used_count: usize,
}

fn read_repositories(base_dir: &Path) -> Result<HashMap<String, String>> {
    let repo_path = base_dir.join("image/overlay2/repositories.json");
    let content = fs::read_to_string(repo_path)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut repositories = HashMap::new();

    if let Value::Object(repos) = &json["Repositories"] {
        for (_, images) in repos {
            if let Value::Object(image_tags) = images {
                for (tag, id) in image_tags {
                    if let Value::String(id_str) = id {
                        repositories.insert(
                            tag.clone(),
                            id_str.trim_start_matches("sha256:").to_string(),
                        );
                    }
                }
            }
        }
    }

    Ok(repositories)
}

fn read_image_contents(base_dir: &Path) -> Result<HashMap<String, Value>> {
    let image_dir = base_dir.join("image/overlay2/imagedb/content/sha256");
    let mut image_contents = HashMap::new();

    for entry in fs::read_dir(image_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path)?;
            let json: Value = serde_json::from_str(&content)?;
            image_contents.insert(path.file_name().unwrap().to_string_lossy().to_string(), json);
        }
    }

    Ok(image_contents)
}

fn read_cache_ids(base_dir: &Path) -> Result<HashMap<String, String>> {
    let layer_dir = base_dir.join("image/overlay2/layerdb/sha256");
    let mut cache_id_map = HashMap::new();

    for entry in fs::read_dir(layer_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let cache_id_path = path.join("cache-id");
            if cache_id_path.exists() {
                let cache_id = fs::read_to_string(cache_id_path)?.trim().to_string();
                cache_id_map.insert(path.file_name().unwrap().to_string_lossy().to_string(), cache_id);
            }
        }
    }

    Ok(cache_id_map)
}fn analyze_overlay(base_dir: &Path) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
    let overlay_dir = base_dir.join("overlay2");
    let mut id_to_short_link = HashMap::new();
    let mut short_link_to_id = HashMap::new();

    for entry in fs::read_dir(overlay_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let id = path.file_name().unwrap().to_string_lossy().to_string();
            let link_path = path.join("link");
            if link_path.exists() {
                let short_link = fs::read_to_string(link_path)?.trim().to_string();
                id_to_short_link.insert(id.clone(), short_link.clone());
                short_link_to_id.insert(short_link, id);
            }
        }
    }

    Ok((id_to_short_link, short_link_to_id))
}

fn build_layer_graph(
    base_dir: &Path,
    id_to_short_link: &HashMap<String, String>,
    short_link_to_id: &HashMap<String, String>,
) -> Result<HashMap<String, Layer>> {
    let mut layer_graph = HashMap::new();

    for (id, short_link) in id_to_short_link {
        let lower_path = base_dir.join("overlay2").join(id).join("lower");
        let mut lower = Vec::new();

        if lower_path.exists() {
            let lower_content = fs::read_to_string(lower_path)?;
            for lower_short_link in lower_content.trim().split(':') {
                let lower_short_link = lower_short_link.trim_start_matches("l/");
                if let Some(lower_id) = short_link_to_id.get(lower_short_link) {
                    lower.push(lower_id.clone());
                }
            }
        }

        layer_graph.insert(id.clone(), Layer {
            short_link: short_link.clone(),
            id: id.clone(),
            lower,
            used_count: 0,
        });
    }

    Ok(layer_graph)
}

fn update_used_count(layer_graph: &mut HashMap<String, Layer>) {
    let layer_ids: Vec<String> = layer_graph.keys().cloned().collect();
    
    for id in layer_ids {
        if let Some(layer) = layer_graph.get(&id) {
            let lower_ids = layer.lower.clone();
            for lower_id in lower_ids {
                if let Some(lower_layer) = layer_graph.get_mut(&lower_id) {
                    lower_layer.used_count += 1;
                }
            }
        }
    }
}
fn find_images_using_layer(
    layer_id: &str,
    image_contents: &HashMap<String, Value>,
    cache_id_map: &HashMap<String, String>
) -> Vec<String> {
    let mut images = Vec::new();

    for (image_id, content) in image_contents {
        if let Some(diff_ids) = content["rootfs"]["diff_ids"].as_array() {
            for diff_id in diff_ids {
                if let Some(diff_id_str) = diff_id.as_str() {
                    let image_layer_id = diff_id_str.trim_start_matches("sha256:");
                    if let Some(overlay_id) = cache_id_map.get(image_layer_id) {
                        if overlay_id == layer_id {
                            images.push(image_id.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    images
}

fn classify_layers(
    repositories: &HashMap<String, String>,
    image_contents: &HashMap<String, Value>,
    cache_id_map: &HashMap<String, String>,
    layer_graph: &HashMap<String, Layer>,
) -> HashMap<String, Vec<String>> {
    let mut classified_layers = HashMap::new();
    classified_layers.insert("valid".to_string(), Vec::new());
    classified_layers.insert("dangling".to_string(), Vec::new());
    classified_layers.insert("orphan".to_string(), Vec::new());

    for (id, layer) in layer_graph {
        if layer.used_count == 0 {
            let images = find_images_using_layer(&layer.id, image_contents, cache_id_map);
            if images.is_empty() {
                println!("Orphan layer: {} ({})", layer.short_link, &layer.id);
                classified_layers.get_mut("orphan").unwrap().push(id.clone());
            } else {
                if repositories.values().any(|v| v == &images[0]) {
                    classified_layers.get_mut("valid").unwrap().push(id.clone());
                } else {
                    classified_layers.get_mut("dangling").unwrap().push(id.clone());
                }
            }
        }
    }

    classified_layers
}

fn display_layer_hierarchy(layer_graph: &HashMap<String, Layer>, classified_layers: &HashMap<String, Vec<String>>) {
    for (category, layers) in classified_layers {
        println!("{}:", category);
        for layer_id in layers {
            display_layer(layer_graph, layer_id, 0, &mut HashSet::new());
        }
        println!();
    }
}

fn display_layer(layer_graph: &HashMap<String, Layer>, layer_id: &str, indent: usize, visited: &mut HashSet<String>) {
    if let Some(layer) = layer_graph.get(layer_id) {
        println!(
            "{}{}+{} (used {} times)",
            "  ".repeat(indent),
            layer.short_link,
            &layer.id[..24],
            layer.used_count
        );

        if visited.insert(layer_id.to_string()) {
            for lower_id in &layer.lower {
                display_layer(layer_graph, lower_id, indent + 1, visited);
            }
            visited.remove(layer_id);
        } else {
            println!("{}  (cyclic reference)", "  ".repeat(indent + 1));
        }
    }
}

fn delete_dangling_files(base_dir: &Path, classified_layers: &HashMap<String, Vec<String>>) -> Result<()> {
    if let Some(dangling_layers) = classified_layers.get("dangling") {
        for layer_id in dangling_layers {
            let image_content_path = base_dir.join(format!("image/overlay2/imagedb/content/sha256/{}", layer_id));
            let overlay_path = base_dir.join(format!("overlay2/{}", layer_id));

            println!("Deleting dangling file: {}", image_content_path.display());
            if let Err(e) = fs::remove_file(&image_content_path) {
                eprintln!("Failed to delete {}: {}", image_content_path.display(), e);
            }

            println!("Deleting dangling directory: {}", overlay_path.display());
            if let Err(e) = fs::remove_dir_all(&overlay_path) {
                eprintln!("Failed to delete {}: {}", overlay_path.display(), e);
            }
        }
    }

    Ok(())
}

// Update the process_docker_files function to use the new methods
pub fn process_docker_files(base_dir: &Path, delete_mode: bool) -> Result<()> {
    let repositories = read_repositories(base_dir)?;
    let image_contents = read_image_contents(base_dir)?;
    let cache_id_map = read_cache_ids(base_dir)?;
    let (id_to_short_link, short_link_to_id) = analyze_overlay(base_dir)?;
    let mut layer_graph = build_layer_graph(base_dir, &id_to_short_link, &short_link_to_id)?;
    update_used_count(&mut layer_graph);
    
    let classified_layers = classify_layers(&repositories, &image_contents, &cache_id_map, &layer_graph);

    // display_layer_hierarchy(&layer_graph, &classified_layers);

    if delete_mode {
        delete_dangling_files(base_dir, &classified_layers)?;
    }

    Ok(())
}