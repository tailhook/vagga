use builder::guard::Guard;
use builder::error::StepError;


pub fn install(pkgs: &Vec<String>, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    guard.ctx.packages.extend(pkgs.clone().into_iter());
    for i in pkgs.iter() {
        guard.ctx.build_deps.remove(i);
    }
    if build {
        try!(guard.distro.install(&mut guard.ctx, pkgs));
    }
    Ok(())
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
