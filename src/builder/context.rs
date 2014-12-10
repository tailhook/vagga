use std::io::IoError;
use std::io::ALL_PERMISSIONS;
use std::io::fs::mkdir_recursive;
use std::collections::TreeSet;

use config::Container;


pub struct BuildContext {
    pub container_name: String,
    pub container_config: Container,
    pub ensure_dirs: TreeSet<Path>,
}

impl BuildContext {
    pub fn new(name: String, container: Container) -> BuildContext {
        return BuildContext {
            container_name: name,
            container_config: container,
            ensure_dirs: vec!(
                Path::new("proc"),
                Path::new("sys"),
                Path::new("dev"),
                Path::new("work"),
                Path::new("tmp"),
                ).into_iter().collect(),
        };
    }

    pub fn finish(&self) -> Result<(), IoError> {
        let base = Path::new("/vagga/root");
        for dir in self.ensure_dirs.iter() {
            try!(mkdir_recursive(&base.join(dir), ALL_PERMISSIONS));
        }
        return Ok(());
    }
}
