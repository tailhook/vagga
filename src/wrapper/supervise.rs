use config::{Config, Settings};
use config::command::SuperviseInfo;

#[allow(unused_args)]
pub fn supervise_cmd(command: &SuperviseInfo, config: &Config,
    settings: &Settings, cmdline: Vec<String>)
    -> Result<int, String>
{
    unimplemented!();
}
