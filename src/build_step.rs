use std::fs;
use std::io::{self, Read};
use std::rc::Rc;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::os::unix::raw::{uid_t, gid_t};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, MetadataExt};


// Convenient reexports
pub use version::{Error as VersionError};
pub use builder::{StepError, Guard};
pub use config::Config;
pub use digest::Digest;

#[derive(Clone, Debug)]
pub struct Step(pub Rc<BuildStep>);


pub trait BuildStep: Debug {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>;
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>;
    fn is_dependent_on(&self) -> Option<&str>;
}

impl BuildStep for Step {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        self.0.hash(cfg, hash)
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        self.0.build(guard, build)
    }
    fn is_dependent_on(&self) -> Option<&str>
    {
        self.is_dependent_on()
    }
}
