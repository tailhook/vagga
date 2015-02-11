use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir_recursive};

use config::builders::Builder;
use config::builders::Builder as B;

use super::context::BuildContext;
use super::commands::debian;
use super::commands::alpine;
use super::commands::generic;
use super::commands::pip;
use super::commands::npm;
use super::tarcmd;
use container::util::clean_dir;
use super::context::Distribution as Distr;


pub trait BuildCommand {
    fn build(&self, ctx: &mut BuildContext) -> Result<(), String>;
}


impl BuildCommand for Builder {
    fn build(&self, ctx: &mut BuildContext) -> Result<(), String> {
        match self {

            &B::Install(ref pkgs) => {
                ctx.packages.extend(pkgs.clone().into_iter());
                for i in pkgs.iter() {
                    ctx.build_deps.remove(i);
                }
                match ctx.distribution {
                    Distr::Unknown => Err(format!("Unknown distribution")),
                    Distr::Ubuntu(_) => debian::apt_install(ctx, pkgs),
                    Distr::Alpine(_) => alpine::install(ctx, pkgs),
                }
            }
            &B::BuildDeps(ref pkgs) => {
                for i in pkgs.iter() {
                    if !ctx.packages.contains(i) {
                        ctx.build_deps.insert(i.clone());
                    }
                }
                match ctx.distribution {
                    Distr::Unknown => Err(format!("Unknown distribution")),
                    Distr::Ubuntu(_) => debian::apt_install(ctx, pkgs),
                    Distr::Alpine(_) => alpine::install(ctx, pkgs),
                }
            }


            &B::Ubuntu(ref name) => {
                debian::fetch_ubuntu_core(ctx, name)
            }
            &B::UbuntuRepo(ref repo) => {
                debian::add_debian_repo(ctx, repo)
            }
            &B::UbuntuUniverse => {
                debian::ubuntu_add_universe(ctx)
            }
            &B::Sh(ref text) => {
                generic::run_command(ctx,
                    &["/bin/sh".to_string(),
                      "-c".to_string(),
                      text.to_string()])
            }
            &B::Cmd(ref cmd) => {
                generic::run_command(ctx, cmd.as_slice())
            }
            &B::Env(ref pairs) => {
                for (k, v) in pairs.iter() {
                    ctx.environ.insert(k.clone(), v.clone());
                }
                Ok(())
            }
            &B::Remove(ref path) => {
                try!(clean_dir(path, true));
                ctx.add_remove_dir(path.clone());
                Ok(())
            }
            &B::EmptyDir(ref path) => {
                try!(clean_dir(path, false));
                ctx.add_empty_dir(path.clone());
                Ok(())
            }
            &B::EnsureDir(ref path) => {
                let fpath = path.path_relative_from(&Path::new("/")).unwrap();
                try!(mkdir_recursive(
                    &Path::new("/vagga/root").join(fpath), ALL_PERMISSIONS)
                    .map_err(|e| format!("Error creating dir: {}", e)));
                ctx.add_ensure_dir(path.clone());
                Ok(())
            }
            &B::CacheDirs(ref pairs) => {
                for (k, v) in pairs.iter() {
                    try!(ctx.add_cache_dir(k.clone(), v.clone()));
                }
                Ok(())
            }
            &B::Depends(_) => {
                Ok(())
            }
            &B::Tar(ref tar) => {
                tarcmd::tar_command(ctx, tar)
            }
            &B::TarInstall(ref tar_inst) => {
                tarcmd::tar_install(ctx, tar_inst)
            }
            &B::Alpine(ref name) => {
                alpine::setup_base(ctx, name)
            }
            &B::PipConfig(ref pip_settings) => {
                ctx.pip_settings = pip_settings.clone();
                Ok(())
            }
            &B::Py2Install(ref pkgs) => {
                pip::pip_install(ctx, 2, pkgs)
            }
            &B::Py3Install(ref pkgs) => {
                pip::pip_install(ctx, 3, pkgs)
            }
            &B::Py2Requirements(ref fname) => {
                pip::pip_requirements(ctx, 2, fname)
            }
            &B::Py3Requirements(ref fname) => {
                pip::pip_requirements(ctx, 3, fname)
            }
            &B::NpmInstall(ref pkgs) => {
                npm::npm_install(ctx, pkgs)
            }
        }
    }
}
