use self::text_section::TextSection;

mod model;
mod parser;
mod text_section;
mod tree;

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

pub fn parse_model(bin: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let data = b"[DATA]";
    let Some(data_start) = bin.windows(data.len()).position(|w| w == data) else {
        Err(anyhow::anyhow!("Data section not found"))?
    };

    let text = String::from_utf8(bin[..data_start].to_vec())?;
    let text_section = TextSection::parse(text)?;

    Ok(())
}
