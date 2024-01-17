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
    pub fn get(&self, index: usize) -> Option<&Window> {
        self.windows.get(index)
    }
    pub fn size(&self) -> usize {
        self.windows.len()
    }
    pub fn max_width(&self) -> usize {
        self.windows.iter().map(Window::width).max().unwrap_or(0) / 2
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

    pub fn iter(&self) -> impl '_ + Iterator<Item = (WindowIndex, f64)> {
        let width = self.width();
        self.coefficients
            .iter()
            .enumerate()
            .zip(std::iter::repeat(width))
            .map(|((idx, coef), width)| (WindowIndex::new(idx, width), *coef))
    }
    pub fn get(&self, index: usize) -> Option<&f64> {
        self.coefficients.get(index)
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
        self.width() - self.left_width() - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowIndex {
    index: usize,
    width: usize,
}

impl WindowIndex {
    pub fn new(index: usize, width: usize) -> Self {
        Self { index, width }
    }

    #[inline(always)]
    pub fn position(&self) -> isize {
        self.index as isize - (self.width / 2) as isize
    }
    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::Window;

    #[test]
    fn width_1() {
        let window = Window::new(vec![0.0]);
        assert_eq!(window.width(), 1);
        assert_eq!(window.left_width(), 0);
        assert_eq!(window.right_width(), 0);
    }

    #[test]
    fn width_3() {
        let window = Window::new(vec![-1.0, 0.0, 1.0]);
        assert_eq!(window.width(), 3);
        assert_eq!(window.left_width(), 1);
        assert_eq!(window.right_width(), 1);
    }

    #[test]
    fn iterator() {
        let window = Window::new(vec![-1.0, 0.0, 1.0]);
        let iterated = window.iter().collect::<Vec<_>>();

        assert_eq!(iterated[0].1, -1.0);
        assert_eq!(iterated[1].1, 0.0);
        assert_eq!(iterated[2].1, 1.0);

        assert_eq!(iterated[0].0.index(), 0);
        assert_eq!(iterated[1].0.index(), 1);
        assert_eq!(iterated[2].0.index(), 2);

        assert_eq!(iterated[0].0.position(), -1);
        assert_eq!(iterated[1].0.position(), 0);
        assert_eq!(iterated[2].0.position(), 1);
    }
}
