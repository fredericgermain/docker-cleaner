use std::rc::Rc;
use std::cell::RefCell;
use crate::analysis::Node;
use crate::overlay2::Overlay2Node;
use crate::image::{ImageLayerNode, ImageContentNode, ImageRepoNode};
use crate::container::ContainerRepoNode;

pub fn display_tree(node: &Rc<RefCell<dyn Node>>, depth: usize) {
    let node = node.borrow();
    let indent = "  ".repeat(depth);

    if let Some(overlay2_node) = node.downcast_ref::<Overlay2Node>() {
        println!("{}Overlay2Node: {} ({})", indent, overlay2_node.short_link, overlay2_node.id);
    } else if let Some(image_layer_node) = node.downcast_ref::<ImageLayerNode>() {
        println!("{}ImageLayerNode: {} ({})", indent, image_layer_node.image_id, image_layer_node.layer_id);
    } else if let Some(image_content_node) = node.downcast_ref::<ImageContentNode>() {
        println!("{}ImageContentNode: {}", indent, image_content_node.image_id);
    } else if let Some(image_repo_node) = node.downcast_ref::<ImageRepoNode>() {
        println!("{}ImageRepoNode: {} ({})", indent, image_repo_node.name_and_tag, image_repo_node.image_id);
    } else if let Some(container_repo_node) = node.downcast_ref::<ContainerRepoNode>() {
        println!("{}ContainerRepoNode: {}", indent, container_repo_node.container_id);
    }

    println!("{}Used count: {}", indent, node.used_count());

    for dep in node.deps() {
        display_tree(dep, depth + 1);
    }
}