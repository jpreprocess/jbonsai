//! Masks unvoiced frames.
//!
//! The unvoiced frames are determined using multi-space probability distribution (MSD) parameter in stream.

use std::ops::Range;

use crate::model::StreamParameter;

/// Mask for unvoiced frames
pub struct Mask {
    // matches the length of `durations`
    ranged_durations: Vec<Range<usize>>,
    // contiguous ranges joined together
    voiced_ranges: Vec<Range<usize>>,
}

impl Mask {
    /// Create mask from `msd` field in stream with lengths of `durations`.
    pub fn create(stream: &StreamParameter, threshold: f64, durations: &[usize]) -> Self {
        let mut i = 0;
        let mut ranged_durations = Vec::with_capacity(durations.len());
        let mut voiced_ranges: Vec<Range<usize>> = Vec::new();
        for ((_, msd), duration) in stream.iter().zip(durations) {
            let range = i..(i + duration);
            if *msd > threshold {
                if let Some(last) = voiced_ranges.last_mut()
                    && last.end == range.start
                {
                    last.end = range.end;
                } else {
                    voiced_ranges.push(range.clone());
                }
            }
            ranged_durations.push(range);
            i += duration;
        }
        Self {
            ranged_durations,
            voiced_ranges,
        }
    }
    pub fn len(&self) -> usize {
        self.ranged_durations.last().map_or(0, |r| r.end)
    }
    /// Get the internal mask.
    pub fn mask(&self) -> Vec<bool> {
        let mut out = vec![false; self.len()];
        for range in &self.voiced_ranges {
            out[range.clone()].fill(true);
        }
        out
    }
    /// Fill back the masked region with `default` and returns an iterator of full-length sequence.
    pub fn fill<'a, T: 'a + Clone>(
        &'a self,
        masked: impl 'a + IntoIterator<Item = T>,
        default: T,
    ) -> impl 'a + Iterator<Item = T> {
        let mut iter = masked.into_iter();
        self.mask().into_iter().map(move |is_voiced| {
            if is_voiced {
                iter.next().unwrap()
            } else {
                default.clone()
            }
        })
    }
    /// Get distances from left- and right-boundaries.
    pub fn boundary_distances(&self) -> Vec<(usize, usize)> {
        if self.ranged_durations.is_empty() {
            return vec![];
        }

        let mut out = vec![(0, 0); self.len()];
        for range in &self.voiced_ranges {
            for i in range.clone() {
                out[i] = (i - range.start, range.end - 1 - i);
            }
        }
        out
    }
}

#[cfg(test)]
#[allow(
    clippy::single_range_in_vec_init,
    reason = "intended vec of single range"
)]
mod tests {
    use super::Mask;

    #[test]
    fn fill() {
        assert_eq!(
            Mask {
                ranged_durations: vec![0..2, 2..4, 4..5, 5..6],
                voiced_ranges: vec![2..4, 5..6]
            }
            .fill([0, 1, 2], 5)
            .collect::<Vec<_>>(),
            vec![5, 5, 0, 1, 5, 2]
        );
        assert_eq!(
            Mask {
                ranged_durations: vec![0..2],
                voiced_ranges: vec![]
            }
            .fill([0, 1], 5)
            .collect::<Vec<_>>(),
            vec![5, 5]
        );
    }
    #[test]
    fn boundary_distances() {
        assert_eq!(
            Mask {
                ranged_durations: vec![0..10],
                voiced_ranges: vec![0..10]
            }
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
            Mask {
                ranged_durations: vec![0..3, 3..5, 5..10],
                voiced_ranges: vec![0..3, 5..10]
            }
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
            Mask {
                ranged_durations: vec![0..3, 3..4, 4..5, 5..10],
                voiced_ranges: vec![0..3, 4..5]
            }
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
        assert_eq!(
            Mask {
                ranged_durations: vec![],
                voiced_ranges: vec![]
            }
            .boundary_distances(),
            vec![]
        );
    }
}
