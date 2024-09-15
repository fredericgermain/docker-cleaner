
fn show_node_details(s: &mut Cursive, node_id: String) {
    let graph = s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
        graph.get(&node_id).cloned()
    }).unwrap();

    if let Some(node) = graph {
        let details = {
            let node = node.borrow();
            format!(
                "ID: {}\nUsed Count: {}\nDependencies: {}",
                node.id(),
                node.used_count(),
                node.deps().len()
            )
        };

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
            s.with_user_data(|graph: &mut HashMap<String, Rc<RefCell<dyn Node>>>| {
                if let Err(e) = remove_node(graph, &node_id, true) {
                    s.add_layer(Dialog::info(format!("Error deleting node: {}", e)));
                } else {
                    s.add_layer(Dialog::info(format!("Node {} deleted successfully", node_id)));
                }
            });
            s.pop_layer();
        }));
}