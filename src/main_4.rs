use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use serde_json::Value;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "docker-cleaner", about = "Clean up dangling Docker files")]
struct Opt {
    #[structopt(short, long, default_value = "/var/lib/docker")]
    base_dir: String,

    #[structopt(short, long)]
    delete: bool,
}

#[derive(Debug, Clone)]
struct Layer {
    sha256: String,
    short_link: String,
    lower: Vec<String>,
    used_count: usize,
}

#[derive(Debug)]
struct ImageTree {
    layers: Vec<Layer>,
    image_name: String,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let base_dir = Path::new(&opt.base_dir);

    let repositories = read_repositories(base_dir)?;
    let valid_images: HashSet<_> = repositories.values().cloned().collect();

    let image_contents = read_image_contents(base_dir, &valid_images)?;
    let (tree_map, valid_trees, dangling_trees, orphan_layers) = analyze_overlay2(base_dir, &repositories, &image_contents)?;

    println!("Valid Image Trees:");
    display_trees(&tree_map, &valid_trees);

    println!("\nDangling Image Trees:");
    display_trees(&tree_map, &dangling_trees);

    println!("\nOrphan Layers:");
  //  for layer in &orphan_layers {
        display_trees(&tree_map, &orphan_layers);
//        println!("  {} ({})", layer.short_link, &layer.sha256[..32]);
    //}

    if opt.delete {
//        delete_dangling_files(base_dir, &dangling_trees, &orphan_layers)?;
    }

    Ok(())
}

fn read_repositories(base_dir: &Path) -> std::io::Result<HashMap<String, String>> {
    let repo_path = base_dir.join("image/overlay2/repositories.json");
    let content = fs::read_to_string(repo_path)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut repositories = HashMap::new();
    if let Value::Object(repos) = &json["Repositories"] {
        for (_, images) in repos {
            if let Value::Object(images) = images {
                for (tag, digest) in images {
                    if let Value::String(digest) = digest {
                        let digest = digest.trim_start_matches("sha256:");
                        repositories.insert(tag.clone(), digest.to_string());
                    }
                }
            }
        }
    }

    Ok(repositories)
}

fn read_image_contents(base_dir: &Path, valid_images: &HashSet<String>) -> std::io::Result<HashSet<String>> {
    let content_dir = base_dir.join("image/overlay2/imagedb/content/sha256");
    let mut contents = HashSet::new();

    for entry in fs::read_dir(content_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap();
        if valid_images.contains(&file_name) {
            contents.insert(file_name);
        }
    }

    Ok(contents)
}

fn analyze_overlay2<'a>(
    base_dir: &Path,
    repositories: &HashMap<String, String>,
    image_contents: &HashSet<String>,
) -> std::io::Result<(HashMap<String, Layer>, Vec<&'a Layer>, Vec<&'a Layer>, Vec<&'a Layer>)> {
    let overlay2_dir = base_dir.join("overlay2");
    let mut layer_sha256_to_short_map = HashMap::new();
    let mut layer_short_to_sha256_map = HashMap::new();
    let mut tree_map = HashMap::new();

    // First pass: build layer_map
    for entry in fs::read_dir(&overlay2_dir)? {
        let entry = entry?;
        let sha256 = entry.file_name().into_string().unwrap();
        let link_path = overlay2_dir.join(&sha256).join("link");
        if link_path.exists() {
            let short_link = fs::read_to_string(link_path)?.trim().to_string();
            // Check for inconsistencies
            if let Some(existing_short) = layer_sha256_to_short_map.get(&sha256) {
                if existing_short != &short_link {
                    println!("Warning: SHA256 {} has multiple short links: {} and {}", sha256, existing_short, short_link);
                    continue;
                }
            }
            if let Some(existing_sha256) = layer_short_to_sha256_map.get(&short_link) {
                if existing_sha256 != &sha256 {
                    println!("Warning: Short link {} has multiple SHA256: {} and {}", short_link, existing_sha256, sha256);
                    continue;
                }
            }

            layer_sha256_to_short_map.insert(sha256.clone(), short_link.clone());
            layer_short_to_sha256_map.insert(short_link, sha256);
        }
    }

    // Second pass: build tree_map
    for (sha256, short_link) in &layer_sha256_to_short_map {
        let lower_path = overlay2_dir.join(&sha256).join("lower");
        let mut lower = Vec::new();
        if lower_path.exists() {
            let lower_content = fs::read_to_string(lower_path)?;
            lower = lower_content
                .split(':')
                .filter_map(|l| layer_short_to_sha256_map.get(l).cloned())
                .collect();
        }
        let layer = Layer {
            sha256: sha256.clone(),
            short_link: short_link.clone(),
            lower,
            used_count: 0,
        };
        tree_map.insert(sha256.clone(), layer);
    }
    
    // Third pass: DFS to fill used_count
    let mut visited = HashSet::new();
    for (root_sha256, _short_link) in &layer_sha256_to_short_map {
        dfs_count(&mut tree_map, root_sha256, &mut visited);
    }

    // Build trees and categorize
    let mut valid_trees = Vec::new();
    let mut dangling_trees = Vec::new();
    let mut orphan_layers = Vec::new();

    for (sha256, layer) in &tree_map {
        if layer.used_count > 0 {
            continue;
        }
      /* let mut layers = Vec::new();
        build_tree(&mut layers, &tree_map, layer);
        let image_tree = ImageTree {
            layers,
            image_name: sha256.clone(),
        }; */
        if repositories.contains_key(sha256) {
            valid_trees.push(layer);
        } else if image_contents.contains(sha256) {
            dangling_trees.push(layer);
        } else {
            orphan_layers.push(layer);
        }
    }

    Ok((tree_map, valid_trees, dangling_trees, orphan_layers))
}

fn build_tree(layers: &mut Vec<Layer>, tree_map: &HashMap<String, Layer>, current: &Layer) {
    layers.push(current.clone());
    for lower_sha256 in &current.lower {
        if let Some(lower_layer) = tree_map.get(lower_sha256) {
            build_tree(layers, tree_map, lower_layer);
        }
    }
}

fn display_trees(tree_map: &HashMap<String, Layer>, layers: &Vec<&Layer>) {
    for layer in layers {
        println!("Image: {}", layers[0].sha256);
        /*
        for (i, layer) in tree.layers.iter().enumerate() {
            let indent = "  ".repeat(i);
            println!(
                "{}{}({}) - Used {} time(s)",
                indent,
                layer.short_link,
                &layer.sha256[..32],
                layer.used_count
            );
        } */
        println!();
    }
}

fn delete_dangling_files(
    base_dir: &Path,
    dangling_trees: &[ImageTree],
    orphan_layers: &[Layer],
) -> std::io::Result<()> {
    let image_content_dir = base_dir.join("image/overlay2/imagedb/content/sha256");
    let overlay2_dir = base_dir.join("overlay2");

    for tree in dangling_trees {
        for layer in &tree.layers {
            let content_file = image_content_dir.join(&layer.sha256);
            let layer_dir = overlay2_dir.join(&layer.short_link);

            if content_file.exists() {
                println!("Delete content file: {:?}", content_file);
                fs::remove_file(content_file)?;
            }

            if layer_dir.exists() {
                println!("Delete layer directory: {:?}", layer_dir);
                fs::remove_dir_all(layer_dir)?;
            }
        }
    }

    for layer in orphan_layers {
        let layer_dir = overlay2_dir.join(&layer.short_link);
        if layer_dir.exists() {
            println!("Delete orphan layer directory: {:?}", layer_dir);
            fs::remove_dir_all(layer_dir)?;
        }
    }

    Ok(())
}

fn dfs_count(tree_map: &mut HashMap<String, Layer>, current_sha256: &str, visited: &mut HashSet<String>) {
    if !visited.insert(current_sha256.to_string()) {
        return; // Already visited this node
    }

    if let Some(layer) = tree_map.get_mut(current_sha256) {
        layer.used_count += 1;
        let lower_sha256s = layer.lower.clone(); // Clone to avoid borrowing issues
        for lower_sha256 in lower_sha256s {
            dfs_count(tree_map, &lower_sha256, visited);
        }
    }
}
