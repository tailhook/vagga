use std::rc::Rc;
use std::fmt::Debug;

use mopa::mopafy;

// Convenient reexports
pub use crate::version::{Error as VersionError};
pub use crate::builder::{StepError, Guard};
pub use crate::config::Config;
pub use crate::digest::Digest;

#[derive(Clone, Debug)]
pub struct Step(pub Rc<dyn BuildStep>);

mopafy!(BuildStep);

pub trait BuildStep: ::mopa::Any + Debug {
    fn name(&self) -> &'static str;
    #[cfg(feature="containers")]
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>;
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>;
    fn is_dependent_on(&self) -> Option<&str>;
}

impl BuildStep for Step {
    fn name(&self) -> &'static str {
        self.0.name()
    }
    #[cfg(feature="containers")]
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        self.0.hash(cfg, hash)
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        self.0.build(guard, build)
    }
    fn is_dependent_on(&self) -> Option<&str>
    {
        self.0.is_dependent_on()
    }
}
