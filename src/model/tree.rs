use std::{collections::HashMap, str::FromStr};

use super::{ModelError, ModelErrorKind};

#[derive(Debug, Clone)]
pub struct TreeNode {
    index: usize,
    question: Question,
    yes: TreeIndex,
    no: TreeIndex,
}

impl TreeNode {
    fn parse(s: &str, questions: &HashMap<String, Question>) -> Result<Self, ModelError> {
        let mut iter = s.split_ascii_whitespace().filter(|s| !s.is_empty());
        let index = iter.next().ok_or(
            ModelErrorKind::TreeNode.with_error(anyhow::anyhow!("Index not found in {}", s)),
        )?;
        let question = iter.next().ok_or(
            ModelErrorKind::TreeNode.with_error(anyhow::anyhow!("Question not found in {}", s)),
        )?;
        let yes = iter.next().ok_or(
            ModelErrorKind::TreeNode
                .with_error(anyhow::anyhow!("'Yes' node id not found in {}", s)),
        )?;
        let no = iter.next().ok_or(
            ModelErrorKind::TreeNode.with_error(anyhow::anyhow!("'No' node id not found in {}", s)),
        )?;
        Ok(Self {
            index: index
                .parse()
                .map_err(|err| ModelErrorKind::TreeNode.with_error(err))?,
            question: questions
                .get(question)
                .ok_or(
                    ModelErrorKind::TreeNode
                        .with_error(anyhow::anyhow!("Question {} not found", question)),
                )?
                .to_owned(),
            yes: TreeIndex::from_str(yes)?,
            no: TreeIndex::from_str(no)?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum TreeIndex {
    Node(usize),
    Pdf(usize),
}

impl FromStr for TreeIndex {
    type Err = ModelError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse() {
            return Ok(Self::Node(id));
        }

        let last_digits = s.split(|c: char| !c.is_ascii_digit()).last().ok_or(
            ModelErrorKind::TreeIndex.with_error(anyhow::anyhow!("Id not found in {}", s)),
        )?;
        let id = last_digits
            .parse()
            .map_err(|err| ModelErrorKind::TreeIndex.with_error(err))?;
        Ok(Self::Pdf(id))
    }
}

#[derive(Debug, Clone)]
pub struct Question {
    name: String,
    patterns: Vec<String>,
}
