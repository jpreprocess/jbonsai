//! Parameters for a stream.

use super::{GvParameter, StreamParameter, Windows};

/// Set of parameters associated with a stream.
pub struct ModelStream<'a> {
    /// The length of parameter vector, which will be generated using the parameters in this struct.
    pub vector_length: usize,
    /// Stream parameter.
    pub stream: StreamParameter,
    /// Global variance parameter.
    pub gv: Option<GvParameter>,
    /// MLPG window coefficients.
    pub windows: &'a Windows,
}
