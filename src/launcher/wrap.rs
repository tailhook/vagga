use std::env;
use std::path::Path;
use std::os::unix::ffi::OsStrExt;

use unshare::{Command, Namespace};

use config::Settings;
use process_util::{convert_status, set_uidmap, copy_env_vars};
use container::uidmap::get_max_uidmap;


pub trait Wrapper {
    fn new(root: Option<&str>, settings: &Settings) -> Self;
    fn workdir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;
    fn userns(&mut self) -> &mut Self;
    fn run(self) -> Result<i32, String>;
}

impl Wrapper for Command {
    fn new(root: Option<&str>, settings: &Settings) -> Self {
        let mut cmd = Command::new("/proc/self/exe");
        cmd.arg0("vagga_wrapper");
        if let Some(root) = root {
            cmd.arg("--root");
            cmd.arg(root);
        };

        cmd.env_clear();

        // Unfortunately OSString does not have starts_with yet
        for (k, v) in env::vars_os() {
            {
                let kbytes = k[..].as_bytes();
                if kbytes.len() < 9 || &kbytes[..9] != &b"VAGGAENV_"[..] {
                    continue
                }
            }
            cmd.env(k, v);
        }
        copy_env_vars(&mut cmd, &settings);
        if let Some(x) = env::var_os("PATH") {
            cmd.env("HOST_PATH", x);
        }
        if let Some(x) = env::var_os("RUST_LOG") {
            cmd.env("RUST_LOG", x);
        }
        if let Some(x) = env::var_os("RUST_BACKTRACE") {
            cmd.env("RUST_BACKTRACE", x);
        }
        if let Some(x) = env::var_os("HOME") {
            cmd.env("VAGGA_USER_HOME", x);
        }

        cmd.unshare(
            [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
        cmd
    }
    fn workdir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.env("PWD", Path::new("/work").join(dir));
        self
    }
    fn userns(&mut self) -> &mut Self {
        set_uidmap(self, &get_max_uidmap().unwrap(), true);
        self
    }
    fn run(mut self) -> Result<i32, String> {
        match self.status() {
            Ok(x) => Ok(convert_status(x)),
            Err(e) => Err(format!("Error running {:?}: {}", self, e)),
        }
    }
}
