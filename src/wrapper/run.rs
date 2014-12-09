use std::rc::Rc;
use std::os::{getcwd, set_exit_status, self_exe_path, getenv};
use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir};
use std::io::fs::PathExtensions;

use container::root::change_root;
use container::mount::{bind_mount, unmount};
use container::monitor::{Monitor, Executor, MonitorStatus, Shutdown};
use container::monitor::{Killed, Exit};
use container::container::{Command};


struct RunCommand {
    cmd: Path,
    args: Vec<String>,
}

impl Executor for RunCommand {
    fn command(&self) -> Command {
        let mut cmd = Command::new("run".to_string(), &self.cmd);
        cmd.args(self.args.as_slice());
        cmd.set_env("TERM".to_string(),
                    getenv("TERM").unwrap_or("dumb".to_string()));
        if let Some(x) = getenv("RUST_LOG") {
            cmd.set_env("RUST_LOG".to_string(), x);
        }
        if let Some(x) = getenv("RUST_BACKTRACE") {
            cmd.set_env("RUST_BACKTRACE".to_string(), x);
        }
        return cmd;
    }
    fn finish(&self, status: int) -> MonitorStatus {
        return Shutdown(status)
    }
}

pub fn run_command(container: String, args: &[String]) -> Result<int, ()> {

    let tgtroot = Path::new("/vagga/root");
    try!(mkdir(&tgtroot, ALL_PERMISSIONS)
         .map_err(|x| error!("Error creating directory: {}", x)));
    try!(bind_mount(&Path::new("/vagga/roots").join(container).join("root"),
                    &tgtroot)
         .map_err(|e| error!("Error bind mount: {}", e)));
    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| error!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| error!("Error unmounting old root: {}", e)));


    let mut mon = Monitor::new();
    let mut cmd = Path::new(args[0].as_slice());
    let args = args[1..].clone().to_vec();
    if cmd.is_absolute() {
    } else {
        let paths = [
            "/bin",
            "/usr/bin",
            "/usr/local/bin",
            "/sbin",
            "/usr/sbin",
            "/usr/local/sbin",
        ];
        let prefix = Path::new("/vagga/root");
        for path in paths.iter() {
            let path = Path::new(*path).join(&cmd);
            if path.exists() {
                cmd = path;
                break;
            }
        }
        if !cmd.is_absolute() {
            error!("Command {} not found in {}",
                cmd.display(), paths.as_slice());
            return Err(());
        }
    }

    mon.add(Rc::new("run".to_string()), box RunCommand {
        cmd: cmd,
        args: args,
    });
    match mon.run() {
        Killed => return Ok(1),
        Exit(val) => return Ok(val),
    };
}
