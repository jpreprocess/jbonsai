pub struct Mask(Vec<bool>);

impl FromIterator<bool> for Mask {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Mask {
    pub fn new(mask: Vec<bool>) -> Self {
        Self(mask)
    }
    pub fn mask(&self) -> &[bool] {
        &self.0
    }
    pub fn fill<T: Clone>(&self, masked: Vec<T>, default: T) -> Vec<T> {
        let mut seq = Vec::with_capacity(self.0.len());
        let mut iter = masked.into_iter();
        for mask in &self.0 {
            if *mask {
                seq.push(iter.next().unwrap());
            } else {
                seq.push(default.clone());
            }
        }
        seq
    }
    pub fn boundary_distances(&self) -> Vec<(usize, usize)> {
        if self.0.is_empty() {
            return vec![];
        }

        let mut result = vec![(0, 0); self.0.len()];

        let mut left = 0;
        for (frame, mask) in self.0.iter().enumerate() {
            if *mask {
                result[frame].0 = frame - left;
            } else {
                // current position will be cut off
                left = frame + 1;
            }
        }

        let mut right = self.0.len() - 1;
        for (frame, mask) in self.0.iter().enumerate().rev() {
            if *mask {
                result[frame].1 = right - frame;
            } else {
                // current position will be cut off
                if frame == 0 {
                    break;
                }
                right = frame - 1;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::sequence::Mask;

    #[test]
    fn boundary_distances() {
        assert_eq!(
            Mask::new(vec![
                true, true, true, true, true, true, true, true, true, true
            ])
            .boundary_distances(),
            vec![
                (0, 9),
                (1, 8),
                (2, 7),
                (3, 6),
                (4, 5),
                (5, 4),
                (6, 3),
                (7, 2),
                (8, 1),
                (9, 0)
            ],
        );
        assert_eq!(
            Mask::new(vec![
                true, true, true, false, false, true, true, true, true, true
            ])
            .boundary_distances(),
            vec![
                (0, 2),
                (1, 1),
                (2, 0),
                (0, 0),
                (0, 0),
                (0, 4),
                (1, 3),
                (2, 2),
                (3, 1),
                (4, 0)
            ]
        );
        assert_eq!(
            Mask::new(vec![
                true, true, true, false, true, false, false, false, false, false
            ])
            .boundary_distances(),
            vec![
                (0, 2),
                (1, 1),
                (2, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0)
            ]
        );
        assert_eq!(Mask::new(vec![]).boundary_distances(), vec![]);
    }
}
