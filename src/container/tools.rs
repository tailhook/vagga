use std::str::FromStr;
use std::borrow::Borrow;


pub trait NextValue {
    fn next_value<T:FromStr>(&mut self) -> Result<T, ()>;
}

impl<I, T: Borrow<str>> NextValue for I
    where I: Iterator<Item=T>
{

    fn next_value<A:FromStr>(&mut self) -> Result<A, ()> {
        self.next().ok_or(())
        .and_then(|x| FromStr::from_str(x.borrow()).map_err(|_| ()))
    }
}

