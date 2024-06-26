//! Masks unvoiced frames.
//!
//! The unvoiced frames are determined using multi-space probability distribution (MSD) parameter in stream.

use crate::model::StreamParameter;

use super::IterExt;

/// Mask for unvoiced frames
pub struct Mask(Vec<bool>);

impl FromIterator<bool> for Mask {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Mask {
    /// Create mask from `msd` field in stream with lengths of `durations`.
    pub fn create(stream: &StreamParameter, threshold: f64, durations: &[usize]) -> Self {
        Self(
            stream
                .iter()
                .map(|(_, msd)| *msd > threshold)
                .duration(durations)
                .collect(),
        )
    }
    /// Get the internal mask.
    pub fn mask(&self) -> &[bool] {
        &self.0
    }
    /// Fill back the masked region with `default` and returns an iterator of full-length sequence.
    pub fn fill<'a, T: 'a + Clone>(
        &'a self,
        masked: impl 'a + IntoIterator<Item = T>,
        default: T,
    ) -> impl 'a + Iterator<Item = T> {
        let mut iter = masked.into_iter();
        self.0.iter().map(move |&mask| {
            if mask {
                iter.next().expect(
                    "Length of `masked` must be the same as the number of `true`'s in mask.",
                )
            } else {
                default.clone()
            }
        })
    }
    /// Get distances from left- and right-boundaries.
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
    use super::Mask;

    #[test]
    fn fill() {
        assert_eq!(
            Mask(vec![false, false, true, true, false, true])
                .fill([0, 1, 2], 5)
                .collect::<Vec<_>>(),
            vec![5, 5, 0, 1, 5, 2]
        );
        assert_eq!(
            Mask(vec![false, false]).fill([0, 1], 5).collect::<Vec<_>>(),
            vec![5, 5]
        );
    }
    #[test]
    fn boundary_distances() {
        assert_eq!(
            Mask(vec![
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
            Mask(vec![
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
            Mask(vec![
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
        assert_eq!(Mask(vec![]).boundary_distances(), vec![]);
    }
}
