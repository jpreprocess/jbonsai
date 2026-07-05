//! Masks unvoiced frames.
//!
//! The unvoiced frames are determined using multi-space probability distribution (MSD) parameter in stream.

use std::ops::Range;

use crate::model::StreamParameter;

/// Mask for unvoiced frames
pub struct Mask {
    // matches the length of `durations`
    pub(super) ranged_durations: Vec<Range<usize>>,
    // contiguous ranges joined together
    pub(super) voiced_ranges: Vec<Range<usize>>,
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
    pub fn voiced_len(&self) -> usize {
        self.voiced_ranges.iter().map(|r| r.len()).sum()
    }
    fn mask(&self) -> Vec<bool> {
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
}

#[derive(Debug)]
pub struct IntoIter<'a> {
    ranged_durations: std::slice::Iter<'a, Range<usize>>,
    voiced_ranges: std::slice::Iter<'a, Range<usize>>,
    next_voiced_range: Option<&'a Range<usize>>,
}

impl<'a> IntoIter<'a> {
    pub fn new(mask: &'a Mask) -> Self {
        let ranged_durations = mask.ranged_durations.iter();
        let mut voiced_ranges = mask.voiced_ranges.iter();
        let next_voiced_range = voiced_ranges.next();
        Self {
            ranged_durations,
            voiced_ranges,
            next_voiced_range,
        }
    }
}

impl Iterator for IntoIter<'_> {
    type Item = (Range<usize>, Option<Range<usize>>);

    fn next(&mut self) -> Option<Self::Item> {
        let range = self.ranged_durations.next()?;
        let voiced_range = self
            .next_voiced_range
            .filter(|r| r.start <= range.start && range.end <= r.end);
        if voiced_range.is_some_and(|r| r.end == range.end) {
            self.next_voiced_range = self.voiced_ranges.next();
        }
        Some((range.clone(), voiced_range.cloned()))
    }
}

impl<'a> IntoIterator for &'a Mask {
    type Item = <IntoIter<'a> as Iterator>::Item;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
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
}
