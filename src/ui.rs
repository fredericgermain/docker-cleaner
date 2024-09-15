use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use cursive::Cursive;
use cursive::views::{Dialog, SelectView, TextView, LinearLayout, ScrollView};
use cursive::traits::*;
use crate::node::Node;
use crate::analysis::{classify_layers, remove_node};
use std::sync::Arc;

pub fn run_ui(mut graph: HashMap<String, Rc<RefCell<dyn Node>>>, delete_mode: bool, dry_run: bool) -> anyhow::Result<()> {
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
    let classified_clone = Arc::clone(&classified);

    let mut select = SelectView::new()
        .on_submit(move |s, item: &String| {
            show_category_details(s, item, Arc::clone(&classified_clone));
        });

    for category in classified.keys() {
        select.add_item(category.clone(), category.clone());
    }

    LinearLayout::vertical()
        .child(TextView::new("Select a category:"))
        .child(select)
}


fn show_category_details(s: &mut Cursive, category: &str, classified: Arc<HashMap<String, Vec<Rc<RefCell<dyn Node>>>>>) {
    let mut nodes: Vec<Rc<RefCell<dyn Node>>> = match classified.get(category) {
        Some(vec) => {
            match category {
                "MissingNode" => vec.clone(),

                _ => vec
                .iter()
                .filter(|node_ref| {
                    let node = node_ref.borrow();
                    node.used_count() == 0
                })
                .cloned() //  .map(Rc::clone)
                .collect(), 

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
