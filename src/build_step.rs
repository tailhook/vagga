use std::rc::Rc;
use std::fmt::Debug;

use shaman::sha2;
use shaman::digest::Digest as DigestTrait;

// Convenient reexports
pub use version::{Error as VersionError};
pub use builder::{StepError, Guard};
pub use config::Config;

#[derive(Clone, Debug)]
pub struct Step(pub Rc<BuildStep>);

pub struct Digest(sha2::Sha256);

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

impl Digest {
    pub fn new() -> Digest {
        Digest(sha2::Sha256::new())
    }
    // TODO(tailhook) get rid of the method
    pub fn unwrap(self) -> sha2::Sha256 {
        return self.0
    }
    pub fn field<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: K, value: V) {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        self.0.input(value.as_ref());
        self.0.input(b"\0");
    }
    pub fn sequence<K, T>(&mut self, key: K, seq: &[T])
        where K: AsRef<[u8]>, T: AsRef<[u8]>
    {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        for value in seq {
            self.0.input(value.as_ref());
            self.0.input(b"\0");
        }
    }
}
