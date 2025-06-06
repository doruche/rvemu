
#[macro_export]
macro_rules! round_up {
    ($val:expr, $align:expr) => {
        (($val as usize + $align as usize - 1) & !($align as usize - 1))        
    };
}

#[macro_export]
macro_rules! round_down {
    ($val:expr, $align:expr) => {
        ($val as usize & !($align as usize - 1))
    };
}