use std::path::Path;
use std::io::{Read, Write};
use std::fs::File;

use libc::getuid;
use unshare::{Command, Stdio};

use crate::launcher::Context;
use crate::process_util::env_path_find;


pub struct SystemInfo {
    pub expect_inotify_limit: Option<usize>,
}


pub fn check(cinfo: &SystemInfo, context: &Context)
    -> Result<(), String>
{
    match cinfo.expect_inotify_limit {
        Some(val) => check_sysctl(context,
            "fs.inotify.max_user_watches", val,
            "http://bit.ly/max_user_watches", 524288),
        None => {}
    }
    Ok(())
}

fn check_sysctl(context: &Context, name: &str, expect: usize,
                link: &str, max: usize) {
    let path = Path::new("/proc/sys").join(name.replace(".", "/"));
    let mut buf = String::with_capacity(10);
    let val: Option<usize> = File::open(&path).ok()
        .and_then(|mut f| f.read_to_string(&mut buf).ok())
        .and_then(|_| buf.trim().parse().ok());
    let real = match val {
        None => {
            warn!("Can't read sysctl {:?}", name);
            return;
        }
        Some(x) => x,
    };

    if real >= expect {
        return;
    }
    if context.settings.auto_apply_sysctl && expect <= max {
        let uid = unsafe { getuid() };
        if uid == 0 {
            File::create(&path)
            .and_then(|mut f| f.write_all(format!("{}", expect).as_bytes()))
            .map_err(|e| error!("Can't apply sysctl {}: {}", name, e)).ok();
        } else if let Some(cmdpath) = env_path_find("sudo") {
            let mut sysctl = Command::new(cmdpath);
            sysctl.stdin(Stdio::null());
            sysctl.arg("-k");
            sysctl.arg("sysctl");
            sysctl.arg(format!("{}={}", name, expect));
            warn!("The sysctl setting {name} is {is} but \
                  at least {expected} is expected. \
                  Running the following command to fix it:\n  \
                    {cmd:?}\n\
                  More info: {link}",
                  name=name, is=real, expected=expect, link=link, cmd=sysctl);
            match sysctl.status() {
                Ok(st) if !st.success() => {
                    error!("Error running sysctl {:?}", st);
                },
                Err(e) => {
                    error!("Error running sysctl: {:?}", e);
                },
                _ => {},
            }
        } else {
            error!("Error running sysctl: `sudo` not found");
        }
    } else {
        warn!("The sysctl setting {name} is {is} but \
              at least {expected} is expected. \
              To fix it till next reboot run:\n  \
                sysctl {name}={expected}\n\
              More info: {link}",
              name=name, is=real, expected=expect, link=link);
        if expect > max {
            warn!("Additionally we can't autofix it \
                   because value is too large. So be careful.")
        }
    }
}
