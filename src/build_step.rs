use std::fmt::Debug;
use version::VersionHash;
use builder::BuildCommand;

pub trait BuildStep: VersionHash + BuildCommand + Debug {
    fn is_dependent_on(&self) -> Option<&str> { None }
}
