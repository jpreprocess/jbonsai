use regex::Regex;

pub struct StreamModels {
    pub metadata: StreamModelMetadata,

    pub stream_model: Model,
    pub gv_model: Option<Model>,
    pub windows: Vec<Vec<f32>>,
}

impl StreamModels {
    pub fn new(
        metadata: StreamModelMetadata,
        stream_model: Model,
        gv_model: Option<Model>,
        windows: Vec<Vec<f32>>,
    ) -> Self {
        StreamModels {
            metadata,
            stream_model,
            gv_model,
            windows,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamModelMetadata {
    pub vector_length: usize,
    pub num_windows: usize,
    pub is_msd: bool,
    pub use_gv: bool,
    pub option: Vec<String>,
}

pub struct Model {
    trees: Vec<Tree>,
    pdf: Vec<Vec<ModelParameter>>,
}

impl Model {
    pub fn new(trees: Vec<Tree>, pdf: Vec<Vec<ModelParameter>>) -> Self {
        Self { trees, pdf }
    }

    /// Get index of tree and PDF
    /// returns (tree_index, pdf_index)
    pub fn get_index(&self, state_index: usize, string: &str) -> (Option<usize>, Option<usize>) {
        let tree_index = self.find_tree_index(state_index, string);

        let tree = match tree_index {
            Some(idx) => &self.trees[idx],
            None => &self.trees[0],
        };

        let pdf_index = tree.search_node(string);

        (
            tree_index
                // Somehow hts_engine_API requires 2 to be added to tree index
                .map(|index| index + 2),
            pdf_index,
        )
    }
    fn find_tree_index(&self, state_index: usize, string: &str) -> Option<usize> {
        self.trees
            .iter()
            .enumerate()
            .position(|(_, tree)| tree.state == state_index && tree.matches_pattern(string))
    }

    /// Get parameter using interpolation weight
    pub fn get_parameter(&self, state_index: usize, string: &str) -> &ModelParameter {
        let (Some(tree_index), Some(pdf_index)) = self.get_index(state_index, string) else {
            todo!("index not found!")
        };

        &self.pdf[tree_index - 2][pdf_index - 1]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelParameter {
    // (mean, vari)
    pub parameters: Vec<(f32, f32)>,
    pub msd: Option<f32>,
}

impl ModelParameter {
    pub fn new(size: usize, is_msd: bool) -> Self {
        Self {
            parameters: vec![(0., 0.); size],
            msd: if is_msd { Some(0.) } else { None },
        }
    }

    pub fn from_linear(lin: Vec<f32>) -> Self {
        let len = lin.len() / 2;
        let mut parameters = Vec::with_capacity(len);
        for i in 0..len {
            parameters.push((lin[i], lin[i + len]))
        }
        Self {
            parameters,
            msd: lin.get(len * 2).copied(),
        }
    }

    pub fn add_assign(&mut self, weight: f32, rhs: &Self) {
        for (i, p) in rhs.parameters.iter().enumerate() {
            self.parameters[i].0 += weight * p.0;
            self.parameters[i].1 += weight * p.1;
        }
        match (self.msd, rhs.msd) {
            (Some(ref mut msd), Some(rhs)) => *msd += weight * rhs,
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub state: usize,
    pub patterns: Vec<Pattern>,
    pub nodes: Vec<TreeNode>,
}

impl Tree {
    /// Pattern match
    #[inline]
    pub fn matches_pattern(&self, string: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(string))
    }
    /// Tree search
    pub fn search_node(&self, string: &str) -> Option<usize> {
        let mut node_index = 0;

        while let Some(node) = self.nodes.get(node_index) {
            match node {
                TreeNode::Leaf { pdf_index } => return Some(*pdf_index),
                TreeNode::Node { patterns, yes, no } => {
                    node_index = if patterns.iter().any(|p| p.is_match(string)) {
                        *yes
                    } else {
                        *no
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub enum TreeNode {
    Node {
        patterns: Vec<Pattern>,
        yes: usize,
        no: usize,
    },
    Leaf {
        pdf_index: usize,
    },
}

#[derive(Debug, Clone)]
pub enum Pattern {
    All,
    Contains(String),
    Regex(Regex),
}

impl Pattern {
    pub fn from_pattern_string<T: AsRef<str>>(pattern: T) -> Result<Self, regex::Error> {
        let pattern = pattern.as_ref();
        if pattern == "*" {
            Ok(Self::All)
        } else if pattern.starts_with('*')
            && pattern.ends_with('*')
            && !pattern[1..pattern.len() - 1].contains(['*', '?'])
        {
            Ok(Self::Contains(pattern[1..pattern.len() - 1].to_string()))
        } else {
            Ok(Self::Regex(Regex::new(&format!(
                "^{}$",
                pattern.replace('*', ".*").replace('?', ".?")
            ))?))
        }
    }
    pub fn is_match(&self, label: &str) -> bool {
        match self {
            Self::All => true,
            Self::Contains(s) => label.contains(s),
            Self::Regex(r) => r.is_match(label),
        }
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::All => matches!(other, Self::All),
            Self::Contains(s1) => matches!(other,Self::Contains(s2) if s1==s2),
            Self::Regex(r1) => matches!(other,Self::Regex(r2) if r1.as_str()==r2.as_str()),
        }
    }
}
