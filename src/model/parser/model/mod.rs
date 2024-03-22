use std::collections::BTreeMap;

use crate::model::voice::{model::Model, question::Question};

use super::{base::ParseTarget, parse_all, ModelParseError};

use self::{
    question::QuestionParser,
    tree::{TreeIndex, TreeParser},
};

mod question;
mod tree;

pub fn parse_model(
    input: &[u8],
    tree_range: (usize, usize),
    pdf_range: (usize, usize),
    pdf_len: usize,
) -> Result<Model, ModelParseError> {
    use nom::{
        combinator::map,
        multi::many_m_n,
        number::complete::{le_f32, le_u32},
        sequence::{pair, terminated},
    };

    let (_, (questions, trees)) = parse_all(
        terminated(
            pair(QuestionParser::parse_questions, TreeParser::parse_trees),
            ParseTarget::sp,
        ),
        tree_range,
    )(input)?;

    let (_, pdf) = parse_all(
        |i| {
            let ntree = trees.len();
            let (mut i, npdf) = many_m_n(ntree, ntree, le_u32)(i)?;
            let mut pdf = Vec::with_capacity(ntree);
            for n in npdf {
                let n = n as usize;
                let (ni, r) = many_m_n(
                    n,
                    n,
                    map(
                        many_m_n(pdf_len, pdf_len, map(le_f32, |v| v as f64)),
                        crate::model::voice::model::ModelParameter::from_linear,
                    ),
                )(i)?;
                pdf.push(r);
                i = ni;
            }
            Ok((i, pdf))
        },
        pdf_range,
    )(input)?;

    let question_lut = BTreeMap::from_iter(questions);
    let new_trees: Vec<_> = trees
        .into_iter()
        .map(|t| convert_tree(t, &question_lut))
        .collect();

    Ok(Model::new(new_trees, pdf))
}

fn convert_tree(
    orig_tree: self::tree::Tree,
    question_lut: &BTreeMap<String, Question>,
) -> crate::model::voice::tree::Tree {
    let node_lut = BTreeMap::from_iter(orig_tree.nodes.iter().enumerate().map(|(i, n)| (n.id, i)));

    if orig_tree.nodes.len() == 1 && orig_tree.nodes[0].yes == orig_tree.nodes[0].no {
        let TreeIndex::Pdf(i) = orig_tree.nodes[0].yes else {
            todo!("Malformed model file. Should not reach here.");
        };
        return crate::model::voice::tree::Tree {
            nodes: vec![crate::model::voice::tree::TreeNode::Leaf {
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

        nodes.push(crate::model::voice::tree::TreeNode::Node {
            question: (*question_lut.get(&node.question_name).unwrap()).clone(),
            yes: yes_id,
            no: no_id,
        });
    }
    nodes.extend(
        pdfs.into_iter()
            .map(|i| crate::model::voice::tree::TreeNode::Leaf {
                pdf_index: i as usize,
            }),
    );

    crate::model::voice::tree::Tree {
        nodes,
        state: orig_tree.state,
    }
}
