#[macro_export]
macro_rules! try_msg {
    ($op:expr, $message:expr) => (
        ($op).map_err(|e| format!($message, err=e))?
    );
    ($op:expr, $message:expr, $($key:ident=$value:expr),*) => (
        ($op).map_err(|e| format!($message, err=e, $($key=$value),*))?
    );
}
