use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir_recursive};
use std::io::fs::PathExtensions;

use config::builders as B;

use super::context::BuildContext;
use super::commands::debian;
use super::commands::generic;
use super::tarcmd;
use container::util::clean_dir;


pub trait BuildCommand {
    fn build(&self, ctx: &mut BuildContext) -> Result<(), String>;
}


impl BuildCommand for B::Builder {
    fn build(&self, ctx: &mut BuildContext) -> Result<(), String> {
        match self {
            &B::UbuntuCore(ref name) => {
                debian::fetch_ubuntu_core(ctx, name)
            }
            &B::UbuntuRepo(ref repo) => {
                debian::add_debian_repo(ctx, repo)
            }
            &B::UbuntuUniverse => {
                debian::ubuntu_add_universe(ctx)
            }
            &B::AptInstall(ref pkgs) => {
                debian::apt_install(ctx, pkgs)
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
            &B::CacheDir(ref pairs) => {
                for (k, v) in pairs.iter() {
                    let path = Path::new("/vagga/root").join(
                        k.path_relative_from(&Path::new("/")).unwrap());
                    if !path.exists() {
                        try!(mkdir_recursive(&path, ALL_PERMISSIONS)
                            .map_err(|e| format!("Can't create dir: {}", e)));
                    }
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
        }
    }
}
