use std::rc::Rc;
use std::fmt::Debug;

// Convenient reexports
pub use version::{Error as VersionError};
pub use builder::{StepError, Guard};
pub use config::Config;
pub use digest::Digest;
pub use capsule::fetch::FetchTask;

#[derive(Clone, Debug)]
pub struct Step(pub Rc<BuildStep>);


pub trait BuildStep: Debug {
    fn name(&self) -> &'static str;
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>;
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>;
    fn is_dependent_on(&self) -> Option<&str>;
    fn get_downloads(&self, _buf: &mut Vec<FetchTask>) { }
}

impl BuildStep for Step {
    fn name(&self) -> &'static str {
        self.0.name()
    }
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
        self.0.is_dependent_on()
    }
    fn get_downloads(&self, buf: &mut Vec<FetchTask>) {
        self.0.get_downloads(buf)
    }
}
