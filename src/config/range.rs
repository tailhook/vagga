use std::fmt;
use libc::uid_t;
use regex::Regex;
use serialize::{Decoder, Decodable};
use std::str::FromStr;


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

impl Decodable for Range {
    fn decode<D:Decoder>(d: &mut D) -> Result<Range, D::Error> {
        match d.read_str() {
            Ok(val) => {
                let num:Option<uid_t> = FromStr::from_str(val.as_slice());
                match num {
                    Some(num) => return Ok(Range::new(num, num)),
                    None => {}
                }
                let regex = Regex::new(r"^(\d+)-(\d+)$").unwrap();
                match regex.captures(val.as_slice()) {
                    Some(caps) => {
                        return Ok(Range::new(
                            caps.at(1).and_then(FromStr::from_str).unwrap(),
                            caps.at(2).and_then(FromStr::from_str).unwrap()));
                    }
                    None => unimplemented!(),
                }
            }
            Err(e) => Err(e),
        }
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

