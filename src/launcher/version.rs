use std::str::CharIndices;
use std::iter::Peekable;
use std::cmp::Ordering;


pub struct Version<'a>(pub &'a str);
pub struct Components<'a>(&'a str, Peekable<CharIndices<'a>>);

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Component<'a> {
    Numeric(u64),
    String(&'a str),
}

impl<'a> Version<'a> {
    fn iter(&self) -> Components<'a> {
        let mut ch = self.0.char_indices().peekable();
        if ch.peek() == Some(&(0, 'v')) {
            ch.next();
        }
        return Components(self.0, ch);
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;
    fn next(&mut self) -> Option<Component<'a>> {
        use self::Component::*;
        while let Some(&(_, x)) = self.1.peek() {
            if x.is_alphanumeric() { break; }
            self.1.next();
        }
        if let Some(&(start, x)) = self.1.peek() {
            if x.is_numeric() {
                while let Some(&(_, x)) = self.1.peek() {
                    if !x.is_numeric() { break; }
                    self.1.next();
                }
                let end = self.1.peek().map(|&(x, _)| x)
                    .unwrap_or(self.0.len());
                let val = &self.0[start..end];
                return Some(val.parse().map(Numeric).unwrap_or(String(val)));
            } else {
                while let Some(&(_, x)) = self.1.peek() {
                    if !x.is_alphanumeric() { break; }
                    self.1.next();
                }
                let end = self.1.peek().map(|&(x, _)| x)
                    .unwrap_or(self.0.len());
                let val = &self.0[start..end];
                return Some(String(val));
            }
        }
        None
    }
}

impl<'a> PartialEq for Version<'a> {
    fn eq(&self, other: &Version) -> bool {
        self.0 == other.0
    }
}


impl<'a> PartialOrd for Version<'a> {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        use self::Component::*;
        use std::cmp::Ordering::*;
        let mut aiter = self.iter();
        let mut biter = other.iter();
        loop {
            let val = match (aiter.next(), biter.next()) {
                (Some(Numeric(x)), Some(Numeric(y))) => x.partial_cmp(&y),
                // git revision starts with g
                (Some(Numeric(_)), Some(String(_))) => Some(Greater),
                (Some(String(_)), Some(Numeric(_))) => Some(Less),
                (Some(String(x)), Some(String(y))) => x.partial_cmp(y),
                (Some(Numeric(_)), None) => Some(Greater),
                (None, Some(Numeric(_))) => Some(Less),
                (None, Some(String(x)))
                if matches!(x, "a"|"b"|"c"|"rc"|"pre"|"dev"|"dirty")
                => Some(Greater),
                (None, Some(String(_))) => Some(Less),
                (Some(String(x)), None)
                if matches!(x, "a"|"b"|"c"|"rc"|"pre"|"dev"|"dirty")
                || x.starts_with("g")
                => Some(Less),
                (Some(String(_)), None) => Some(Greater),
                (None, None) => Some(Equal),
            };
            if val != Some(Equal) {
                return val;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Version;

    #[test]
    fn test_version_parse() {
        use super::Component::*;
        assert_eq!(Version("v0.4.1-28-gfba00d7").iter().collect::<Vec<_>>(),
            [Numeric(0), Numeric(4), Numeric(1),
             Numeric(28), String("gfba00d7")]);
    }

    #[test]
    fn test_version_cmp() {
        assert!(Version("v0.4.1-28-gfba00d7") > Version("v0.4.1"));
        assert!(Version("v0.4.1-28-gfba00d7") > Version("v0.4.1-27-gtest"));
        assert!(Version("v0.4.1-28-gfba00d7") < Version("v0.4.2"));
    }
}

