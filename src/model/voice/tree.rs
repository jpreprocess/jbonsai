use jlabel::Label;

use super::question::Question;

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    pub state: usize,
    pub nodes: Vec<TreeNode>,
}

impl Tree {
    /// Tree search
    pub fn search_node(&self, label: &Label) -> Option<usize> {
        let mut node_index = 0;

        while let Some(node) = self.nodes.get(node_index) {
            match node {
                TreeNode::Leaf { pdf_index } => return Some(*pdf_index),
                TreeNode::Node { question, yes, no } => {
                    node_index = if question.test(label) { *yes } else { *no }
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeNode {
    Node {
        question: Question,
        yes: usize,
        no: usize,
    },
    Leaf {
        pdf_index: usize,
    },
}
