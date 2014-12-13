use std::io::ALL_PERMISSIONS;
use std::io::fs::mkdir_recursive;

use config::builders as B;

use super::context::BuildContext;
use super::commands::debian;
use super::commands::generic;
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
            &B::Sh(ref text) => {
                generic::run_command(ctx,
                    &["/bin/sh".to_string(),
                      "-c".to_string(),
                      text.to_string()])
            }
            &B::Cmd(ref cmd) => {
                generic::run_command(ctx, cmd.as_slice())
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
        }
    }
}
