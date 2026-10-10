macro_rules! twice {
    ($e:expr) => {
        $e * 2
    };
}

pub fn raw(p: *const u8) -> u8 {
    unsafe { *p }
}

pub fn ok() -> i32 {
    42
}
