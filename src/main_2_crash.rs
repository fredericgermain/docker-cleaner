use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use serde_json::Value;
use structopt::StructOpt;
use walkdir::WalkDir;
use std::io::{self, Write};

#[derive(StructOpt, Debug)]
#[structopt(name = "docker-cleaner")]
struct Opt {
    #[structopt(short, long, default_value = "/var/lib/docker")]
    base_dir: String,

    #[structopt(short, long)]
    delete: bool,
}

#[derive(Debug, Clone)]
struct LayerInfo {
    sha256: String,
    short_link: String,
    use_count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let base_dir = Path::new(&opt.base_dir);

    let repositories = read_repositories(base_dir)?;
    let image_contents = read_image_contents(base_dir)?;
    let (valid_trees, dangling_trees, orphan_trees) = analyze_overlay2(base_dir, &repositories, &image_contents)?;

    println!("Valid image trees:");
    for tree in &valid_trees {
        display_tree(tree, 0);
    }

    println!("\nDangling image trees:");
    for tree in &dangling_trees {
        display_tree(tree, 0);
    }

    println!("\nOrphan trees:");
    for tree in &orphan_trees {
        display_tree(tree, 0);
    }

    if opt.delete {
        delete_dangling_data(base_dir, &dangling_trees, &image_contents)?;
    }

    Ok(())
}

fn read_repositories(base_dir: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let repo_file = base_dir.join("image/overlay2/repositories.json");
    let content = fs::read_to_string(repo_file)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut repositories = HashSet::new();
    if let Value::Object(obj) = json {
        for (_key, value) in obj {
            if let Value::Object(inner_obj) = value {
                for (_tag, sha) in inner_obj {
                    if let Value::String(sha_str) = sha {
                        repositories.insert(sha_str);
                    }
                }
            }
        }
    }

    Ok(repositories)
}

fn read_image_contents(base_dir: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let content_dir = base_dir.join("image/overlay2/imagedb/content/sha256");
    let mut contents = HashSet::new();

    for entry in fs::read_dir(content_dir)? {
        let entry = entry?;
        contents.insert(entry.file_name().into_string().unwrap());
    }

    Ok(contents)
}

fn analyze_overlay2(
    base_dir: &Path,
    repositories: &HashSet<String>,
    image_contents: &HashSet<String>,
) -> Result<(Vec<LayerInfo>, Vec<LayerInfo>, Vec<LayerInfo>), Box<dyn std::error::Error>> {
    let overlay2_dir = base_dir.join("overlay2");
    let mut valid_trees = Vec::new();
    let mut dangling_trees = Vec::new();
    let mut orphan_trees = Vec::new();

    let mut layer_map: HashMap<String, LayerInfo> = HashMap::new();
    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(&overlay2_dir).min_depth(1).max_depth(1) {
        let entry = entry?;
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        
        let link_path = entry.path().join("link");
        let sha256 = if link_path.exists() {
            fs::read_to_string(link_path)?.trim().to_string()
        } else {
            dir_name.clone()
        };

        let layer_info = LayerInfo {
            sha256: sha256.clone(),
            short_link: dir_name.clone(),
            use_count: 0,
        };
        layer_map.insert(sha256.clone(), layer_info);

        if let Ok(lower) = fs::read_to_string(entry.path().join("lower")) {
            let parents: Vec<String> = lower.trim().split(':').map(String::from).collect();
            tree_map.insert(sha256, parents);
        } else {
            tree_map.insert(sha256, Vec::new());
        }
    }

    // Count uses
    for (_, parents) in &tree_map {
        for parent in parents {
            if let Some(layer_info) = layer_map.get_mut(parent) {
                layer_info.use_count += 1;
            }
        }
    }

    for (sha256, layer_info) in &layer_map {
        let tree = reconstruct_tree(sha256, &tree_map, &layer_map);
        if repositories.contains(sha256) {
            valid_trees.push(tree);
        } else if image_contents.contains(sha256) {
            dangling_trees.push(tree);
        } else {
            orphan_trees.push(tree);
        }
    }

    Ok((valid_trees, dangling_trees, orphan_trees))
}

fn reconstruct_tree(root: &str, tree_map: &HashMap<String, Vec<String>>, layer_map: &HashMap<String, LayerInfo>) -> LayerInfo {
    let mut layer_info = layer_map.get(root).unwrap().clone();
    if let Some(children) = tree_map.get(root) {
        layer_info.use_count = children.len();
        for child in children {
            let child_tree = reconstruct_tree(child, tree_map, layer_map);
            layer_info.use_count += child_tree.use_count;
        }
    }
    layer_info
}

fn display_tree(layer_info: &LayerInfo, indent: usize) {
    let indent_str = " ".repeat(indent);
    println!("{}{}:{} (used {} times)", 
             indent_str, 
             layer_info.short_link, 
             &layer_info.sha256[..32], 
             layer_info.use_count);
}

fn delete_dangling_data(
    base_dir: &Path,
    dangling_trees: &[LayerInfo],
    image_contents: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nDeleting dangling data:");

    for layer_info in dangling_trees {
        // Delete content in image/overlay2/imagedb/content/sha256
        let content_file = base_dir.join(format!("image/overlay2/imagedb/content/sha256/{}", layer_info.sha256));
        if image_contents.contains(&layer_info.sha256) && content_file.exists() {
            if confirm_delete(&content_file)? {
                fs::remove_file(&content_file)?;
                println!("Deleted {}", content_file.display());
            }
        }

        // Delete directory in overlay2
        let overlay2_dir = base_dir.join(format!("overlay2/{}", layer_info.short_link));
        if overlay2_dir.exists() {
            if confirm_delete(&overlay2_dir)? {
                fs::remove_dir_all(&overlay2_dir)?;
                println!("Deleted {}", overlay2_dir.display());
            }
        }
    }

    Ok(())
}

fn confirm_delete(path: &Path) -> Result<bool, io::Error> {
    print!("Delete {}? [y/N] ", path.display());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}
