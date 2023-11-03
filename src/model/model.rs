use regex::Regex;

pub struct StreamModels {
    stream_model: Model,
    gv_model: Option<Model>,
    windows: Vec<Vec<f32>>,
}

impl StreamModels {
    pub fn new(stream_model: Model, gv_model: Option<Model>, windows: Vec<Vec<f32>>) -> Self {
        StreamModels {
            stream_model,
            gv_model,
            windows,
        }
    }
}

pub struct Model {
    trees: Vec<Tree>,
    pdf: Vec<Vec<ModelParameter>>,
}

impl Model {
    pub fn new(trees: Vec<Tree>, pdf: Vec<Vec<ModelParameter>>) -> Self {
        Self { trees, pdf }
    }
}

pub struct ModelParameter {
    // (mean, vari)
    parameters: Vec<(f32, f32)>,
    msd: Option<f32>,
}

impl ModelParameter {
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
}

pub struct Tree {
    pub state: usize,
    pub patterns: Vec<Pattern>,
    pub nodes: Vec<TreeNode>,
}

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

pub enum Pattern {
    All,
    Contains(String),
    Regex(Regex),
}

impl Pattern {
    pub fn from_pattern_strings<T: AsRef<str>>(pattern: T) -> Result<Self, regex::Error> {
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
