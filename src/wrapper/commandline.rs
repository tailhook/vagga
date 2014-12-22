use config::{Config, Settings};
use config::command::CommandInfo;

#[allow(unused_args)]
pub fn commandline_cmd(command: &CommandInfo, config: &Config,
    settings: &Settings, cmdline: Vec<String>)
    -> Result<int, String>
{
    unimplemented!();
}
