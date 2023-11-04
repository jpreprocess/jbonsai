use std::collections::BTreeMap;

use crate::model::model::Pattern;

use super::tree::{Tree, TreeIndex};

fn parse_patterns(patterns: &[String]) -> Result<Vec<crate::model::model::Pattern>, regex::Error> {
    patterns
        .iter()
        .map(crate::model::model::Pattern::from_pattern_string)
        .collect()
}

pub fn convert_tree(
    mut orig_tree: Tree,
    question_lut: &BTreeMap<&String, &Vec<Pattern>>,
) -> crate::model::model::Tree {
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

    orig_tree.nodes.sort_by_key(|n| n.id);

    let mut pdfs = Vec::new();
    for node in &orig_tree.nodes {
        match node.yes {
            TreeIndex::Pdf(id) => pdfs.push(id),
            _ => (),
        }
        match node.no {
            TreeIndex::Pdf(id) => pdfs.push(id),
            _ => (),
        }
    }
    pdfs.sort_unstable();

    let mut nodes = Vec::new();
    for node in &orig_tree.nodes {
        let yes_id = match node.yes {
            TreeIndex::Node(id) => orig_tree.nodes.binary_search_by_key(&id, |k| k.id),
            TreeIndex::Pdf(id) => pdfs.binary_search(&id).map(|v| v + orig_tree.nodes.len()),
        }
        .unwrap();
        let no_id = match node.no {
            TreeIndex::Node(id) => orig_tree.nodes.binary_search_by_key(&id, |k| k.id),
            TreeIndex::Pdf(id) => pdfs.binary_search(&id).map(|v| v + orig_tree.nodes.len()),
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
