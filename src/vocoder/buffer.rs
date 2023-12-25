pub use std::ops::{Index, IndexMut};
use std::ops::{RangeFrom, RangeFull};
pub use std::slice::SliceIndex;

pub trait Buffer:
    Index<usize, Output = f64>
    + IndexMut<usize, Output = f64>
    + Index<RangeFull, Output = [f64]>
    + Index<RangeFrom<usize>, Output = [f64]>
    + IndexMut<RangeFrom<usize>, Output = [f64]>
{
    fn len(&self) -> usize;
    fn iter(&self) -> <&Vec<f64> as IntoIterator>::IntoIter;
}

macro_rules! buffer_index {
    ($t:ty) => {
        impl<I: SliceIndex<[f64]>> Index<I> for $t {
            type Output = I::Output;

            fn index(&self, index: I) -> &Self::Output {
                &self.buffer[index]
            }
        }

        impl<I: SliceIndex<[f64]>> IndexMut<I> for $t {
            fn index_mut(&mut self, index: I) -> &mut Self::Output {
                &mut self.buffer[index]
            }
        }

        impl<'a> IntoIterator for &'a $t {
            type Item = &'a f64;
            type IntoIter = <&'a Vec<f64> as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.buffer.iter()
            }
        }

        impl Buffer for $t {
            fn len(&self) -> usize {
                self.buffer.len()
            }

            #[allow(dead_code)]
            fn iter(&self) -> <&Vec<f64> as IntoIterator>::IntoIter {
                self.buffer.iter()
            }
        }
    };
}
