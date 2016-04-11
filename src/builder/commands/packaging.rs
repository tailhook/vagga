use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


// Build Steps
#[derive(Debug)]
pub struct Install(Vec<String>);
tuple_struct_decode!(Install);

impl BuildStep for Install {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
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

pub fn build_deps(pkgs: &Vec<String>, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    for i in pkgs.iter() {
        if !guard.ctx.packages.contains(i) {
            guard.ctx.build_deps.insert(i.clone());
        }
    }
    if build {
        try!(guard.distro.install(&mut guard.ctx, pkgs));
    }
    Ok(())
}
