#[macro_export]
macro_rules! try_msg {
    ($op:expr, $message:expr) => (
        ($op).map_err(|e| format!($message, err=e))?
    );
    ($op:expr, $message:expr, $($key:ident=$value:expr),*) => (
        ($op).map_err(|e| format!($message, err=e, $($key=$value),*))?
    );
}

#[macro_export]
macro_rules! tuple_struct_decode {
    ($name:ident) => {
        impl ::rustc_serialize::Decodable for $name {
            fn decode<D: ::rustc_serialize::Decoder>(d: &mut D)
                -> Result<$name, D::Error>
            {
                ::rustc_serialize::Decodable::decode(d).map($name)
            }
        }
    }
}
