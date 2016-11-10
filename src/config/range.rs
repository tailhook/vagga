use std::str::FromStr;

use libc::uid_t;
use rustc_serialize::{Decoder, Decodable};


#[derive(Clone, Debug, Copy)]
pub struct Range {
    start: uid_t,
    end: uid_t,
}

trait StringError<T> {
    fn create_error(&self, value: String) -> T;
}

impl Decodable for Range {
    fn decode<D:Decoder>(d: &mut D) -> Result<Range, D::Error>
    {
        d.read_str().and_then(|val| {
            FromStr::from_str(&val[..])
            .map(|num| Range::new(num, num))
            .or_else(|_| {
                let mut pair = val.splitn(2, '-');
                Ok(Range::new(
                    pair.next().and_then(|x| FromStr::from_str(x).ok())
                        .ok_or(d.error("Error parsing range"))?,
                    pair.next().and_then(|x| FromStr::from_str(x).ok())
                        .ok_or(d.error("Error parsing range"))?,
                ))
            })
        })
    }
}

impl Range {
    pub fn new(start: uid_t, end: uid_t) -> Range {
        assert!(end >= start);
        return Range { start: start, end: end+1 };
    }
    pub fn len(&self) -> uid_t {
        return self.end - self.start;
    }
    pub fn shift(&self, val: uid_t) -> Range {
        assert!(self.end - self.start >= val);
        return Range { start: self.start + val, end: self.end };
    }
    pub fn start(&self) -> uid_t {
        self.start
    }
    pub fn end(&self) -> uid_t {
        self.end + 1
    }
}

