use std::collections::BTreeMap;

use crate::model::model::Pattern;

use super::tree::{Tree, TreeIndex};

pub fn convert_tree(
    orig_tree: Tree,
    question_lut: &BTreeMap<&String, &Vec<Pattern>>,
) -> crate::model::model::Tree {
    let node_lut = BTreeMap::from_iter(orig_tree.nodes.iter().enumerate().map(|(i, n)| (n.id, i)));

    if orig_tree.nodes.len() == 1 && orig_tree.nodes[0].yes == orig_tree.nodes[0].no {
        let TreeIndex::Pdf(i) = orig_tree.nodes[0].yes else {
            todo!("Malformed model file. Should not reach here.");
        };
        return crate::model::model::Tree {
            patterns: orig_tree.pattern,
            nodes: vec![crate::model::model::TreeNode::Leaf {
                pdf_index: i as usize,
            }],
            state: orig_tree.state,
        };
    }

    let mut pdfs = Vec::new();
    for node in &orig_tree.nodes {
        if let TreeIndex::Pdf(id) = node.yes {
            pdfs.push(id)
        }
        if let TreeIndex::Pdf(id) = node.no {
            pdfs.push(id)
        }
    }
    pdfs.sort_unstable();

    let mut nodes = Vec::new();
    for node in &orig_tree.nodes {
        let yes_id = match node.yes {
            TreeIndex::Node(id) => node_lut.get(&id).copied(),
            TreeIndex::Pdf(id) => pdfs
                .binary_search(&id)
                .map(|v| v + orig_tree.nodes.len())
                .ok(),
        }
        .unwrap();
        let no_id = match node.no {
            TreeIndex::Node(id) => node_lut.get(&id).copied(),
            TreeIndex::Pdf(id) => pdfs
                .binary_search(&id)
                .map(|v| v + orig_tree.nodes.len())
                .ok(),
        }
        .unwrap();

        nodes.push(crate::model::model::TreeNode::Node {
            patterns: question_lut.get(&node.question_name).unwrap().to_vec(),
            yes: yes_id,
            no: no_id,
        });
    }
    nodes.extend(
        pdfs.into_iter()
            .map(|i| crate::model::model::TreeNode::Leaf {
                pdf_index: i as usize,
            }),
    );

    crate::model::model::Tree {
        patterns: orig_tree.pattern,
        nodes,
        state: orig_tree.state,
    }
}
