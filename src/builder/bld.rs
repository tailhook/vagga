use config::builders as B;

use super::context::BuildContext;
use super::commands::debian;


pub trait BuildCommand {
    fn build(&self, ctx: &mut BuildContext) -> Result<(), String>;
}


impl BuildCommand for B::Builder {
    fn build(&self, ctx: &mut BuildContext) -> Result<(), String> {
        match self {
            &B::UbuntuCore(ref name) => {
                debian::fetch_ubuntu_core(ctx, name)
            }
            _ => unimplemented!(),
        }
    }
}
