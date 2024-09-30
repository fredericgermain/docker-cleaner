use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use cursive::Cursive;
use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, SelectView, TextView, LinearLayout, ScrollView};
use crate::node::Node;
use crate::analysis::{classify_layers, remove_node, remove_node_list};

// Define a struct to hold all your user data
#[allow(dead_code)]
struct UiAppState {
    base_path: PathBuf,
    graph: HashMap<String, Rc<RefCell<dyn Node>>>,
}

struct UiMainNode {
    pub desc: &'static str,
    pub node_type: &'static str,
}

// Define the static list for the main nodes outside of main
static UPPER_NODES: &[UiMainNode] = &[
    UiMainNode { desc: "Images", node_type: "ImageRepo" },
    UiMainNode { desc: "Containers", node_type: "Container" },
    //UiMainNode { desc: "Volumes", node_type: "Mount" },
   // UiMainNode { desc: "Networks", node_type: "network" },
];

// Define the static list for the main nodes outside of main
static DANDLING_NODES: &[UiMainNode] = &[
    UiMainNode { desc: "MetadataDiffId", node_type: "MetadataDiffId" },
    UiMainNode { desc: "LayerDiffId", node_type: "LayerDiffId" },
    UiMainNode { desc: "ImageContent", node_type: "ImageContent" },
    UiMainNode { desc: "ImageLayer", node_type: "ImageLayer" },
    UiMainNode { desc: "Overlay2", node_type: "Overlay2" },
    UiMainNode { desc: "Mount", node_type: "Mount" },

    UiMainNode { desc: "Images", node_type: "ImageRepo" },
];

pub fn run_ui(graph: HashMap<String, Rc<RefCell<dyn Node>>>, base_path: PathBuf) -> anyhow::Result<()> {
    let mut siv = cursive::default();

    let _app_state = UiAppState {
        base_path,
        graph: graph.clone(),
    };

    siv.set_user_data(graph);
    siv.add_layer(Dialog::around(build_main_view())
        .title("Docker Cleaner")
        .button("Quit", |s| s.quit()));

    siv.run();

    Ok(())
}

fn build_main_view() -> impl View {

    // Create the "Upper node" section
    let mut upper_select = SelectView::new()
        .on_submit(move |s, item: &str| {
            show_category_details(s, item, false);
        });

    for node in UPPER_NODES.iter() {
        upper_select.add_item(node.desc, node.node_type);
    }

    let mut dangling_select = SelectView::new()
        .on_submit(move |s, item: &str| {
            show_category_details(s, item, true);
        });

    for node in DANDLING_NODES.iter() {
        dangling_select.add_item(node.desc, node.node_type);
    }

    // Create the "Missing node" section
    let missing_select = SelectView::new()
        .item("Missing nodes", "MissingNode")
        .on_submit(move |s, item: &str| {
            show_category_details(s, item, false);
        });

    return LinearLayout::vertical()
        .child( TextView::new(StyledString::styled(
            "Upper level nodes",
            Style::from(Effect::Bold).combine(Effect::Underline).combine(Color::Dark(BaseColor::Red)),
        )))
        .child(upper_select)
        .child(TextView::new("                                 "))
        .child( TextView::new(StyledString::styled(
            "Dangling nodes",
            Style::from(Effect::Bold).combine(Effect::Underline).combine(Color::Dark(BaseColor::Red)),
        )))
        .child(dangling_select)
        .child(TextView::new("                                 "))
        .child( TextView::new(StyledString::styled(
            "Missing nodes",
            Style::from(Effect::Bold).combine(Effect::Underline).combine(Color::Dark(BaseColor::Red)),
        )))
        .child(missing_select)
        .child(TextView::new("                                 "));
}


fn show_category_details(s: &mut Cursive, category: &str, dandling: bool) {

    let classified = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
        classify_layers(&graph)
    }).unwrap();

    let mut nodes: Vec<Rc<RefCell<dyn Node>>> = match classified.get(category) {
        Some(vec) => {
            if dandling {
                vec
                .iter()
                .filter(|node_ref| {
                    let node = node_ref.borrow();
                    node.rdeps().len() == 0
                })
                .cloned() //  .map(Rc::clone)
                .collect()
            } else {
                vec.clone()
            }
        }
        
        None => Vec::new()
    };
    nodes
        .sort_by_key(|node| node.borrow().id());

    let mut select = SelectView::new()
        .on_submit(move |s, item: &String| {
            show_node_details(s, item.clone());
        });

    for node in &nodes {
        let node_id = node.borrow().id();
        select.add_item(node_id.clone(), node_id);
    }

    s.add_layer(Dialog::around(ScrollView::new(select))
        .title(format!("{} Details", category))
        .button("Back", |s| { s.pop_layer(); }));
}

fn show_node_details(s: &mut Cursive, node_id: String) {
    let node = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
        Rc::clone(graph.get(&node_id).unwrap())
    });

    if let Some(node) = node {
        let node = node;
        let details = format!(
            "ID: {}",
            node.borrow().id()
        );

        let mut dependencies_select = SelectView::new()
        .on_submit(move |s, node_id: &str| {
            let node_ref = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
                Rc::clone(graph.get(node_id).unwrap())
            }).unwrap();
            show_node_details(s, node_ref.borrow().id());
        });
        for dep_node in node.borrow().deps().iter() {
            dependencies_select.add_item(dep_node.borrow().id(), dep_node.borrow().id());
        }
        let mut rdependencies_select = SelectView::new()
        .on_submit(move |s, node_id: &str| {
            let node_ref = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
                Rc::clone(graph.get(node_id).unwrap())
            }).unwrap();
            show_node_details(s, node_ref.borrow().id());
        });
        for rdep_node in node.borrow().rdeps().iter() {
            rdependencies_select.add_item(rdep_node.borrow().id(), rdep_node.borrow().id());
        }
        let view = LinearLayout::vertical()
            .child(TextView::new(details))
            .child(TextView::new("                                 "))
            .child( TextView::new(StyledString::styled(
                format!("Dependencies ({})", node.borrow().deps().len()),
                Style::from(Effect::Bold).combine(Effect::Underline).combine(Color::Dark(BaseColor::Red)),
            )))
            .child(dependencies_select)
            .child(TextView::new("                                 "))
            .child( TextView::new(StyledString::styled(
                format!("Reverse dependencies ({})", node.borrow().rdeps().len()),
                Style::from(Effect::Bold).combine(Effect::Underline).combine(Color::Dark(BaseColor::Red)),
            )))
            .child(rdependencies_select)
            .child(TextView::new("                                 "));

        let mut node_detail = Dialog::around(view)
            .title("Node Details")
            .button("Back", |s| { s.pop_layer(); });
        let hard_deps_count = node.borrow().deps().iter().fold(0, |acc, node| {
            if !node.borrow().id().starts_with("Missing") {
                acc + 1
            } else {
                acc
            }
        });
        if node.borrow().rdeps().len() == 0 {
            let node2 = Rc::clone(&node);
            node_detail = node_detail.button("Delete", move |s| {
                delete_node(s, Rc::clone(&node2), false);
            });
            if hard_deps_count > 0 {
                let node3 = Rc::clone(&node);
                node_detail = node_detail.button("Delete Recursive", move |s| {
                    delete_node(s, Rc::clone(&node3), true);
                });
            }
        }
        s.add_layer(node_detail);
    }
}

fn delete_node(s: &mut Cursive, node: Rc<RefCell<dyn Node>>, recursive: bool) {

    let dep_list = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
        remove_node_list(graph, &node.borrow().id(), recursive)
    }).unwrap().unwrap();

    let result = dep_list.iter()
    .fold(String::new(), |mut acc, item| {
        acc.push_str("\n - ");
        acc.push_str(&item.borrow().id());
        acc
    });

    s.add_layer(Dialog::around(TextView::new(format!("Are you sure you want to delete {} and deps ?\n{}", node.borrow().id(), result)))
        .title("Confirm Deletion")
        .button("Cancel", |s| { s.pop_layer(); })
        .button("Delete", move |s| {
            let result = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
                remove_node(graph, Rc::clone(&node), recursive)
            }).unwrap();

            s.pop_layer();

            match result {
                Ok(_) => {
                    s.pop_layer();
                    s.add_layer(Dialog::info(format!("Node {} deleted successfully", node.borrow().id())));
                },
                Err(e) => {
                    s.add_layer(Dialog::info(format!("Error deleting node: {}", e)));
                }
            }
        }));
}
