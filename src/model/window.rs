#[derive(Debug, Clone, PartialEq)]
pub struct Windows {
    windows: Vec<Window>,
}

impl Windows {
    pub fn new(windows: Vec<Window>) -> Self {
        Self { windows }
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = &Window> {
        self.windows.iter()
    }
    pub fn size(&self) -> usize {
        self.windows.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    coefficients: Vec<f64>,
}

impl Window {
    pub fn new(coefficients: Vec<f64>) -> Self {
        Self { coefficients }
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = (isize, f64)> {
        let zero_index = (self.coefficients.len() / 2) as isize;
        self.coefficients
            .iter()
            .enumerate()
            .zip(std::iter::repeat(zero_index))
            .map(|((idx, coef), zero_index)| (idx as isize - zero_index, *coef))
    }

    #[inline(always)]
    pub fn width(&self) -> usize {
        self.coefficients.len()
    }
    #[inline(always)]
    pub fn left_width(&self) -> usize {
        self.width() / 2
    }
    #[inline(always)]
    pub fn right_width(&self) -> usize {
        self.width() - self.left_width()
    }
}
