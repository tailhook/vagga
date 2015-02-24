use std::fmt;
use libc::uid_t;
use serialize::{Decoder, Decodable};
use std::str::FromStr;
use std::error::FromError;
use quire::decode::YamlDecoder;


#[derive(Clone, Show, Copy)]
pub struct Range {
    pub start: uid_t,
    pub end: uid_t,
}

impl fmt::String for Vec<Range> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "["));
        let mut iter = self.iter();
        if let Some(i) = iter.next() {
            try!(write!(fmt, "{}-{}", i.start, i.start+i.end));
        }
        for i in iter {
            try!(write!(fmt, ", {}-{}", i.start, i.start+i.end));
        }
        try!(write!(fmt, "]"));
        Ok(())
    }
}

struct RangeError;

trait StringError<T> {
    fn create_error(&self, value: String) -> T;
}

impl Decodable for Range {
    fn decode<D:Decoder>(d: &mut D) -> Result<Range, D::Error>
    {
        d.read_str().and_then(|val| {
            FromStr::from_str(val.as_slice())
            .map(|num| Range::new(num, num))
            .or_else(|_| {
                let mut pair = val.splitn(1, '-');
                Ok(Range::new(
                    try!(pair.next().and_then(|x| FromStr::from_str(x).ok())
                        .ok_or(d.error("Error parsing range"))),
                    try!(pair.next().and_then(|x| FromStr::from_str(x).ok())
                        .ok_or(d.error("Error parsing range"))),
                ))
            })
        })
    }
}

impl Range {
    pub fn new(start: uid_t, end: uid_t) -> Range {
        return Range { start: start, end: end };
    }
    pub fn len(&self) -> uid_t {
        return self.end - self.start + 1;
    }
    pub fn shift(&self, val: uid_t) -> Range {
        assert!(self.end - self.start + 1 >= val);
        return Range::new(self.start + val, self.end);
    }
}

