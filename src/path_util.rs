use std::path::{Path, PathBuf};
use std::path::Component::RootDir;

trait ToRelative {
    fn rel<'x>(&'x self) -> &'x Path;
}

impl ToRelative for Path {
    fn rel<'x>(&'x self) -> &'x Path {
        let mut iter = self.components();
        assert!(iter.next() == Some(RootDir));
        iter.as_path()
    }
}

impl ToRelative for PathBuf {
    fn rel<'x>(&'x self) -> &'x Path {
        let mut iter = self.components();
        assert!(iter.next() == Some(RootDir));
        iter.as_path()
    }
}
