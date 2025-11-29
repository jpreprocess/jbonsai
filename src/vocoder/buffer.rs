pub use std::ops::{Deref, DerefMut};

pub trait Buffer: Deref<Target = [f64]> + DerefMut {}

macro_rules! deref_buffer {
    ($t:ty) => {
        impl Deref for $t {
            type Target = [f64];

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
