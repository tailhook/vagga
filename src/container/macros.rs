#![macro_use]

#[macro_export]
macro_rules! try_str {
    ($expr:expr) => {
        try!(($expr).map_err(|e| format!("{}: {}", stringify!($expr), e)))
    }
}

#[macro_export]
macro_rules! try_opt {
    ($expr:expr) => {
        match $expr {
            Some(x) => x,
            None => return None,
        }
    }
}

