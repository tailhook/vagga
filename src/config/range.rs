use libc::uid_t;
use serialize::{Decoder, Decodable};
use std::from_str::FromStr;


#[deriving(Clone, Show)]
pub struct Range {
    pub start: uid_t,
    pub end: uid_t,
}

impl<E, D:Decoder<E>> Decodable<D, E> for Range {
    fn decode(d: &mut D) -> Result<Range, E> {
        match d.read_str() {
            Ok(val) => {
                let num:Option<uid_t> = FromStr::from_str(val.as_slice());
                match num {
                    Some(num) => return Ok(Range::new(num, num)),
                    None => {}
                }
                match regex!(r"^(\d+)-(\d+)$").captures(val.as_slice()) {
                    Some(caps) => {
                        return Ok(Range::new(
                            from_str(caps.at(1)).unwrap(),
                            from_str(caps.at(2)).unwrap()));
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

