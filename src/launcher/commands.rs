use std::io::{self, stdout, stderr};
use std::fs::{read_link};
use std::env;
use std::path::Path;

use argparse::{ArgumentParser, StoreConst};

use crate::file_util::{force_symlink, safe_ensure_dir};
use crate::launcher::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    All,
    Local,
    User,
}


pub fn update_symlinks(ctx: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    use self::Which::*;

    let mut which = All;
    {
        args.insert(0, "vagga _update_symlinks".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Update symlinks in `.vagga/.cmd` and `~/.vagga/cmd`
            ");
        ap.refer(&mut which)
            .add_option(&["--local-only"], StoreConst(Local),
                "Only update local shell scripts (`.vagga/.cmd`)")
            .add_option(&["--user-only"], StoreConst(User), "
                Only update shell script in user's home (`$HOME/.vagga/cmd`)");
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    if !ctx.settings.run_symlinks_as_commands {
        warn!("To make these symlinks useful, \
            you should enable: `run-symlinks-as-commands` setting");
    }

    if which == All || which == Local {
        update_local_links(ctx)?;
    }
    if which == All || which == User {
        update_user_links(ctx)?;
    }

    Ok(0)
}

fn update_user_links(ctx: &Context) -> Result<(), String> {

    let home = if let Ok(home) = env::var("_VAGGA_HOME") {
        home
    } else {
        return Err("No HOME environment variable defined".to_string());
    };

    let vagga = Path::new(&home).join(".vagga");
    safe_ensure_dir(&vagga)?;
    let cmddir = vagga.join("cmd");
    safe_ensure_dir(&cmddir)?;
    let vagga_exe = read_link("/proc/self/exe")
        .map_err(|e| format!("can't find vagga's executable: {}", e))?;

    for (_, ref cmd) in &ctx.config.commands {
        if let Some(link) = cmd.link() {
            let dest = cmddir.join(link.name);
            match read_link(&dest) {
                Ok(ref value) if value == &vagga_exe => continue,
                Ok(_) => {},
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("read_link error: {:?}", e)),
            };
            force_symlink(&vagga_exe, &dest)
                .map_err(|e| format!("Error symlinking: {}", e))?;
        }
    }

    Ok(())
}

fn update_local_links(ctx: &Context) -> Result<(), String> {
    let vagga = ctx.config_dir.join(".vagga");
    safe_ensure_dir(&vagga)?;
    let cmddir = vagga.join(".cmd");
    safe_ensure_dir(&cmddir)?;
    let vagga_exe = read_link("/proc/self/exe")
        .map_err(|e| format!("can't find vagga's executable: {}", e))?;

    for (_, ref cmd) in &ctx.config.commands {
        if let Some(link) = cmd.link() {
            let dest = cmddir.join(link.name);
            match read_link(&dest) {
                Ok(ref value) if value == &vagga_exe => continue,
                Ok(_) => {},
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("read_link error: {:?}", e)),
            };
            force_symlink(&vagga_exe, &dest)
                .map_err(|e| format!("Error symlinking: {}", e))?;
        }
    }

    Ok(())
}

