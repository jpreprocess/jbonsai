pub use std::ops::{Deref, DerefMut};

pub trait Buffer: Deref<Target = Vec<f64>> + DerefMut {}

macro_rules! buffer_index {
    ($t:ty) => {
        impl Deref for $t {
            type Target = Vec<f64>;

            fn deref(&self) -> &Self::Target {
                &self.buffer
            }
        }

        impl DerefMut for $t {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.buffer
            }
        }

        impl Buffer for $t {}
    };
}
