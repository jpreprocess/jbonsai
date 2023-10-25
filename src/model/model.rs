use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Clone)]
pub struct Model {
    pub vector_length: usize,
    pub num_windows: usize,
    pub is_msd: bool,
    pub ntree: usize,
    pub npdf: Vec<usize>,
    pub pdf: Vec<f64>,
}

impl Model {
    pub fn new(
        mut data: impl Read,
        ntree: usize,
        vector_length: usize,
        num_windows: usize,
        is_msd: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let npdf = (0..ntree)
            .into_iter()
            .map(|_| data.read_u64::<LittleEndian>().map(|v| v as usize))
            .collect::<Result<_, _>>()?;

        let pdf_len = ntree * ntree * (vector_length * num_windows * 2 + (is_msd as usize));
        let pdf = (0..pdf_len)
            .into_iter()
            .map(|_| data.read_f64::<LittleEndian>())
            .collect::<Result<_, _>>()?;

        Ok(Self {
            vector_length,
            num_windows,
            is_msd,
            ntree,
            npdf,
            pdf,
        })
    }
}
