use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use cursive::Cursive;
use cursive::views::{Dialog, SelectView, TextView, LinearLayout, ScrollView};
use cursive::traits::*;
use crate::node::Node;
use crate::analysis::{classify_layers, find_dangling_nodes, remove_node};

pub fn run_ui(mut graph: HashMap<String, Rc<RefCell<dyn Node>>>, delete_mode: bool, dry_run: bool) -> anyhow::Result<()> {
    let mut siv = cursive::default();

    let classified = classify_layers(&graph);
    let dangling = find_dangling_nodes(&graph);

    siv.add_layer(Dialog::around(build_main_view(&classified, &dangling))
        .title("Docker Cleaner")
        .button("Quit", |s| s.quit()));

    siv.run();

    Ok(())
}

fn build_main_view(classified: &HashMap<String, Vec<Rc<RefCell<dyn Node>>>>, dangling: &[Rc<RefCell<dyn Node>>]) -> impl View {
    let mut select = SelectView::new()
        .on_submit(move |s, item: &String| {
            show_category_details(s, item, classified, dangling);
        });

    for category in classified.keys() {
        select.add_item(category.clone(), category.clone());
    }
    select.add_item("Dangling Nodes".to_string(), "Dangling Nodes".to_string());

    LinearLayout::vertical()
        .child(TextView::new("Select a category:"))
        .child(select)
}

fn show_category_details(s: &mut Cursive, category: &str, classified: &HashMap<String, Vec<Rc<RefCell<dyn Node>>>>, dangling: &[Rc<RefCell<dyn Node>>]) {
    let nodes = if category == "Dangling Nodes" {
        dangling.to_vec()
    } else {
        classified.get(category).cloned().unwrap_or_default()
    };

    let mut select = SelectView::new()
        .on_submit(move |s, item: &String| {
            show_node_details(s, item);
        });

    for node in &nodes {
        let node_id = node.borrow().id();
        select.add_item(node_id.clone(), node_id);
    }

    s.add_layer(Dialog::around(ScrollView::new(select))
        .title(format!("{} Details", category))
        .button("Back", |s| { s.pop_layer(); }));
}

fn show_node_details(s: &mut Cursive, node_id: &str) {
    s.call_on_name("graph", move |graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
        if let Some(node) = graph.get(node_id) {
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
                    delete_node(s, node_id);
                }));
        }
    });
}

fn delete_node(s: &mut Cursive, node_id: &str) {
    s.add_layer(Dialog::around(TextView::new(format!("Are you sure you want to delete {}?", node_id)))
        .title("Confirm Deletion")
        .button("Cancel", |s| { s.pop_layer(); })
        .button("Delete", move |s| {
            s.call_on_name("graph", move |graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
                if let Err(e) = remove_node(graph, node_id, true) {
                    s.add_layer(Dialog::info(format!("Error deleting node: {}", e)));
                } else {
                    s.add_layer(Dialog::info(format!("Node {} deleted successfully", node_id)));
                }
            });
            s.pop_layer();
        }));
}