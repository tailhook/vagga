use config::builders as B;

use super::context::BuildContext;
use super::commands::debian;
use super::commands::generic;


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
        }
    }
}
