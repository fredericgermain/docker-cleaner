use serde_json::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Layer {
    short_link: String,
    id: String, // actually the sha256
    lower: RefCell<Vec<Rc<Layer>>>,
    used_count: usize,
}

pub fn analyze_docker_files(base_dir: PathBuf, delete_mode: bool) -> io::Result<()> {
    let repositories = read_repositories(&base_dir)?;
    let image_contents = read_image_contents(&base_dir)?;
    let (valid, dangling, orphan) = analyze_overlay(&base_dir, &repositories, &image_contents)?;

    println!("Valid images:");
    display_tree(&valid);

    println!("\nDangling images:");
    display_tree(&dangling);

    println!("\nOrphan layers:");
    display_tree(&orphan);

    if delete_mode {
        delete_dangling_files(&base_dir, &dangling, &orphan)?;
    }

    Ok(())
}

fn read_repositories(base_dir: &Path) -> io::Result<HashMap<String, String>> {
    let repo_path = base_dir.join("image/overlay2/repositories.json");
    let repo_content = fs::read_to_string(repo_path)?;
    let repo_json: Value = serde_json::from_str(&repo_content)?;

    let mut repositories = HashMap::new();
    if let Some(repos) = repo_json["Repositories"].as_object() {
        for (_, tags) in repos {
            if let Some(tags) = tags.as_object() {
                for (tag, sha) in tags {
                    let sha = sha.as_str().unwrap_or_default().trim_start_matches("sha256:");
                    repositories.insert(tag.to_string(), sha.to_string());
                }
            }
        }
    }

    Ok(repositories)
}

fn read_image_contents(base_dir: &Path) -> io::Result<HashSet<String>> {
    let content_dir = base_dir.join("image/overlay2/imagedb/content/sha256");
    let mut image_contents = HashSet::new();

    for entry in fs::read_dir(content_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        image_contents.insert(file_name);
    }

    Ok(image_contents)
}

fn analyze_overlay(
    base_dir: &Path,
    repositories: &HashMap<String, String>,
    image_contents: &HashSet<String>,
) -> io::Result<(Vec<Rc<Layer>>, Vec<Rc<Layer>>, Vec<Rc<Layer>>)> {
    let overlay_dir = base_dir.join("overlay2");
    let (layer_map_id_to_short_link, layer_map_short_link_to_id) = build_layer_maps(&overlay_dir)?;

    let mut layers = HashMap::new();
    for (id, short_link) in &layer_map_id_to_short_link {
        let layer = Rc::new(Layer {
            short_link: short_link.clone(),
            id: id.clone(),
            lower: RefCell::new(Vec::new()),
            used_count: 0,
        });
        layers.insert(id.clone(), layer);
    }

    build_layer_tree(&overlay_dir, &layers, &layer_map_short_link_to_id)?;

    let (valid, dangling, orphan) = categorize_layers(&layers, repositories, image_contents);

    Ok((valid, dangling, orphan))
}

fn build_layer_maps(overlay_dir: &Path) -> io::Result<(HashMap<String, String>, HashMap<String, String>)> {
    let mut id_to_short_link = HashMap::new();
    let mut short_link_to_id = HashMap::new();

    for entry in fs::read_dir(overlay_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        if file_name.len() == 64 {
            let link_file = entry.path().join("link");
            if link_file.exists() {
                let short_link = fs::read_to_string(link_file)?.trim().to_string();
                id_to_short_link.insert(file_name.clone(), short_link.clone());
                short_link_to_id.insert(short_link, file_name);
            }
        }
    }

    Ok((id_to_short_link, short_link_to_id))
}

fn build_layer_tree(
    overlay_dir: &Path,
    layers: &HashMap<String, Rc<Layer>>,
    short_link_to_id: &HashMap<String, String>,
) -> io::Result<()> {
    let mut queue = VecDeque::new();
    for layer in layers.values() {
        queue.push_back(layer.clone());
    }

    while let Some(layer) = queue.pop_front() {
        let lower_file = overlay_dir.join(&layer.id).join("lower");
        if lower_file.exists() {
            let lower_content = fs::read_to_string(lower_file)?;
            for lower_short_link in lower_content.split(':') {
                let lower_short_link = lower_short_link.trim_start_matches("l/");
                if let Some(lower_id) = short_link_to_id.get(lower_short_link) {
                    if let Some(lower_layer) = layers.get(lower_id) {
                        layer.lower.borrow_mut().push(lower_layer.clone());
                        lower_layer.clone().used_count += 1;
                        queue.push_back(lower_layer.clone());
                    }
                }
            }
        }
    }

    Ok(())
}

fn categorize_layers(
    layers: &HashMap<String, Rc<Layer>>,
    repositories: &HashMap<String, String>,
    image_contents: &HashSet<String>,
) -> (Vec<Rc<Layer>>, Vec<Rc<Layer>>, Vec<Rc<Layer>>) {
    let mut valid = Vec::new();
    let mut dangling = Vec::new();
    let mut orphan = Vec::new();

    for layer in layers.values() {
        if layer.lower.borrow().is_empty() {
            if repositories.values().any(|v| v == &layer.id) {
                valid.push(layer.clone());
            } else if image_contents.contains(&layer.id) {
                dangling.push(layer.clone());
            } else {
                orphan.push(layer.clone());
            }
        }
    }

    (valid, dangling, orphan)
}

fn display_tree(layers: &[Rc<Layer>]) {
    for layer in layers {
        display_layer(layer, 0);
    }
}

fn display_layer(layer: &Rc<Layer>, depth: usize) {
    let indent = "  ".repeat(depth);
    println!(
        "{}{}:{} (used: {})",
        indent,
        layer.short_link,
        &layer.id[..32],
        layer.used_count
    );
    for lower in layer.lower.borrow().iter() {
        display_layer(lower, depth + 1);
    }
}

fn delete_dangling_files(base_dir: &Path, dangling: &[Rc<Layer>], orphan: &[Rc<Layer>]) -> io::Result<()> {
    for layer in dangling.iter().chain(orphan.iter()) {
        let image_file = base_dir.join(format!("image/overlay2/imagedb/content/sha256/{}", layer.id));
        let overlay_dir = base_dir.join(format!("overlay2/{}", layer.id));

        if image_file.exists() {
            println!("Delete {}? (y/n)", image_file.display());
            if confirm_deletion() {
                fs::remove_file(image_file)?;
                println!("Deleted {}", image_file.display());
            }
        }

        if overlay_dir.exists() {
            println!("Delete {}? (y/n)", overlay_dir.display());
            if confirm_deletion() {
                fs::remove_dir_all(overlay_dir)?;
                println!("Deleted {}", overlay_dir.display());
            }
        }
    }

    Ok(())
}

fn confirm_deletion() -> bool {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}