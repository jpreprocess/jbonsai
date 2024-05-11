use super::{GvParameter, StreamParameter, Windows};

pub struct ModelStream<'a> {
    pub vector_length: usize,
    pub stream: StreamParameter,
    pub gv: Option<GvParameter>,
    pub windows: &'a Windows,
}
