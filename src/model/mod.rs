mod parser;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModelErrorKind {
    TreeNode,
    TreeIndex,
}

impl ModelErrorKind {
    pub fn with_error<E>(self, source: E) -> ModelError
    where
        anyhow::Error: From<E>,
    {
        ModelError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ModelError(kind={kind:?}, source={source})")]
pub struct ModelError {
    pub kind: ModelErrorKind,
    source: anyhow::Error,
}
