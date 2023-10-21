pub type size_t = libc::c_ulong;
pub type HTS_Boolean = libc::c_char;
pub type uint32_t = libc::c_uint;

#[derive(Copy, Clone)]
pub struct HTS_File {
    pub type_0: libc::c_uchar,
    pub pointer: *mut libc::c_void,
}

#[macro_export]
macro_rules! HTS_error {
    ($error:expr,$message:expr,$args:expr,) => {};
    ($error:expr,$message:expr,$args:expr) => {};
    ($error:expr,$message:expr) => {};
    ($error:expr,$message:expr,) => {};
}

pub const MAX_F0: f64 = 20000.0;
pub const MIN_F0: f64 = 20.0;
pub const MAX_LF0: f64 = 9.903_487_552_536_127; /* log(20000.0) */
pub const MIN_LF0: f64 = 2.995_732_273_553_991; /* log(20.0) */
pub const HALF_TONE: f64 = 0.057_762_265_046_662_11; /* log(2.0) / 12.0 */
pub const DB: f64 = 0.115_129_254_649_702_28; /* log(10.0) / 20.0 */
