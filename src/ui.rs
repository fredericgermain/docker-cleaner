use std::collections::{ HashMap, HashSet };
use std::rc::Rc;
use std::cell::RefCell;
use cursive::Cursive;
use cursive::views::{Dialog, SelectView, TextView, LinearLayout, ScrollView};
use cursive::traits::*;
use crate::node::Node;
use crate::analysis::{classify_layers, remove_node};
use std::sync::Arc;

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
    UiMainNode { desc: "DiffId", node_type: "DiffId" },
    UiMainNode { desc: "ImageContent", node_type: "ImageContent" },
    UiMainNode { desc: "ImageLayer", node_type: "ImageLayer" },
    UiMainNode { desc: "Overlay2", node_type: "Overlay2" },
    UiMainNode { desc: "Volumes", node_type: "Mount" },

    UiMainNode { desc: "Images", node_type: "ImageRepo" },
];

pub fn run_ui(graph: HashMap<String, Rc<RefCell<dyn Node>>>) -> anyhow::Result<()> {
    let mut siv = cursive::default();

    let classified = Arc::new(classify_layers(&graph));

    siv.set_user_data(graph);
    siv.add_layer(Dialog::around(build_main_view(Arc::clone(&classified)))
        .title("Docker Cleaner")
        .button("Quit", |s| s.quit()));

    siv.run();

    Ok(())
}

fn build_main_view(classified: Arc<HashMap<String, Vec<Rc<RefCell<dyn Node>>>>>) -> impl View {
    let classified_clone1 = Arc::clone(&classified);

    // Create the "Upper node" section
    let mut upper_select = SelectView::new()
        .on_submit(move |s, item: &str| {
            show_category_details(s, item, Arc::clone(&classified_clone1), false);
        });

    for node in UPPER_NODES.iter() {
        upper_select.add_item(node.desc, node.node_type);
    }

    let classified_clone2 = Arc::clone(&classified);
    let mut dandling_select = SelectView::new()
        .on_submit(move |s, item: &String| {
            show_category_details(s, item, Arc::clone(&classified_clone2), true);
        });


    let main_node_types: HashSet<&str> = UPPER_NODES.iter().map(|node| node.node_type).collect();

    let classified_clone2 = Arc::clone(&classified);
    let mut remaining_node_types: Vec<String> = classified_clone2.keys()
        .filter(|node_type| !main_node_types.contains(node_type.as_str()))
        .cloned()
        .collect();

    remaining_node_types.sort();

    for category in remaining_node_types {
        dandling_select.add_item(category.clone(), category.clone());
    }

    // Create the "Missing node" section
    let classified_clone3 = Arc::clone(&classified);
    let missing_select = SelectView::new()
        .item("Missing nodes", "MissingNode")
        .on_submit(move |s, item: &str| {
            show_category_details(s, item, Arc::clone(&classified_clone3), false);
        });


    return LinearLayout::vertical()
        .child(Dialog::around(upper_select).title("Upper level nodes"))
        .child(Dialog::around(dandling_select).title("Dandling nodes"))
        .child(Dialog::around(missing_select).title("Missing nodes"));
}


fn show_category_details(s: &mut Cursive, category: &str, classified: Arc<HashMap<String, Vec<Rc<RefCell<dyn Node>>>>>, dandling: bool) {
    let mut nodes: Vec<Rc<RefCell<dyn Node>>> = match classified.get(category) {
        Some(vec) => {
            if dandling {
                vec
                .iter()
                .filter(|node_ref| {
                    let node = node_ref.borrow();
                    node.used_count() == 0
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
    let graph = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
        graph.get(&node_id).cloned()
    }).unwrap();

    if let Some(node) = graph {
        let node = node.borrow();
        let details = format!(
            "ID: {}\nUsed Count: {}\nDependencies: {}",
            node.id(),
            node.used_count(),
            node.deps().len()
        );

        s.add_layer(Dialog::around(TextView::new(details))
            .title("Node Details")
            .button("Back", |s| { s.pop_layer(); })
            .button("Delete", move |s| {
                delete_node(s, node_id.clone());
            }));
    }
}

fn delete_node(s: &mut Cursive, node_id: String) {
    s.add_layer(Dialog::around(TextView::new(format!("Are you sure you want to delete {}?", node_id)))
        .title("Confirm Deletion")
        .button("Cancel", |s| { s.pop_layer(); })
        .button("Delete", move |s| {
            let result = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
                remove_node(graph, &node_id, true)
            }).unwrap();

            match result {
                Ok(_) => {
                    s.add_layer(Dialog::info(format!("Node {} deleted successfully", node_id)));
                },
                Err(e) => {
                    s.add_layer(Dialog::info(format!("Error deleting node: {}", e)));
                }
            }
            s.pop_layer();
        }));
}
