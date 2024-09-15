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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let base_dir = Path::new(&opt.base_dir);

    let repositories = read_repositories(base_dir)?;
    let image_contents = read_image_contents(base_dir)?;
    let (valid_trees, dangling_trees, orphan_trees) = analyze_overlay2(base_dir, &repositories, &image_contents)?;

    println!("Valid image trees:");
    for tree in &valid_trees {
        println!("  {}", tree);
    }

    println!("\nDangling image trees:");
    for tree in &dangling_trees {
        println!("  {}", tree);
    }

    println!("\nOrphan trees:");
    for tree in &orphan_trees {
        println!("  {}", tree);
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
) -> Result<(Vec<String>, Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let overlay2_dir = base_dir.join("overlay2");
    let mut valid_trees = Vec::new();
    let mut dangling_trees = Vec::new();
    let mut orphan_trees = Vec::new();

    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(&overlay2_dir).min_depth(1).max_depth(1) {
        let entry = entry?;
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        
        if let Ok(lower) = fs::read_to_string(entry.path().join("lower")) {
            let parents: Vec<String> = lower.trim().split(':').map(String::from).collect();
            tree_map.insert(dir_name.clone(), parents);
        } else {
            tree_map.insert(dir_name.clone(), Vec::new());
        }
    }

    for (root, _) in &tree_map {
        let tree = reconstruct_tree(root, &tree_map);
        if repositories.contains(root) {
            valid_trees.push(tree);
        } else if image_contents.contains(root) {
            dangling_trees.push(tree);
        } else {
            orphan_trees.push(tree);
        }
    }

    Ok((valid_trees, dangling_trees, orphan_trees))
}

fn reconstruct_tree(root: &str, tree_map: &HashMap<String, Vec<String>>) -> String {
    let mut tree = root.to_string();
    if let Some(children) = tree_map.get(root) {
        for child in children {
            tree.push_str(" -> ");
            tree.push_str(&reconstruct_tree(child, tree_map));
        }
    }
    tree
}

fn delete_dangling_data(
    base_dir: &Path,
    dangling_trees: &[String],
    image_contents: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nDeleting dangling data:");

    for tree in dangling_trees {
        let root = tree.split_whitespace().next().unwrap();
        
        // Delete content in image/overlay2/imagedb/content/sha256
        let content_file = base_dir.join(format!("image/overlay2/imagedb/content/sha256/{}", root));
        if image_contents.contains(root) && content_file.exists() {
            if confirm_delete(&content_file)? {
                fs::remove_file(&content_file)?;
                println!("Deleted {}", content_file.display());
            }
        }

        // Delete directory in overlay2
        let overlay2_dir = base_dir.join(format!("overlay2/{}", root));
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

