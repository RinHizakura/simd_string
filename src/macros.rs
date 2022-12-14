#[macro_export]
macro_rules! align_up {
    ($val: expr, $align:expr) => {
        ($val) + ((!($val) + 1) & (($align) - 1))
    };
}

#[macro_export]
macro_rules! align_down {
    ($val: expr, $align:expr) => {
        ($val) & !(($align) - 1)
    };
}
