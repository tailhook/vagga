use quire::validate as V;

use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


// Build Steps
#[derive(Debug)]
pub struct Install(Vec<String>);
tuple_struct_decode!(Install);

impl Install {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Debug)]
pub struct BuildDeps(Vec<String>);
tuple_struct_decode!(BuildDeps);

impl BuildDeps {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

impl BuildStep for Install {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("Install", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        guard.ctx.packages.extend(self.0.clone().into_iter());
        for i in self.0.iter() {
            guard.ctx.build_deps.remove(i);
        }
        if build {
            try!(guard.distro.install(&mut guard.ctx, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for BuildDeps {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("BuildDeps", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            for i in self.0.iter() {
                if !guard.ctx.packages.contains(i) {
                    guard.ctx.build_deps.insert(i.clone());
                }
            }
            try!(guard.distro.install(&mut guard.ctx, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
