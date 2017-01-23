use std::io::{stdout, stderr, Write, BufWriter};
use std::fs::{File, Permissions, rename, set_permissions};
use std::env;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use launcher::Context;

use argparse::{ArgumentParser, StoreConst};

use file_util::{safe_ensure_dir};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    All,
    Local,
    User,
}


pub fn update_commands(ctx: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    use self::Which::*;

    let mut which = All;
    {
        args.insert(0, "vagga _update_commands".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Update scripts in `.vagga/cmd` and `~/.vagga/cmd`
            ");
        ap.refer(&mut which)
            .add_option(&["--local-only"], StoreConst(Local),
                "Only update local shell scripts (`.vagga/cmd`)")
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

    if which == All || which == Local {
        update_local_scripts(ctx)?;
    }
    if which == All || which == User {
        update_user_scripts(ctx)?;
    }

    Ok(0)
}

fn update_user_scripts(ctx: &Context) -> Result<(), String> {

    let home = if let Ok(home) = env::var("_VAGGA_HOME") {
        home
    } else {
        return Err("No HOME environment variable defined".to_string());
    };

    let vagga = Path::new(&home).join(".vagga");
    safe_ensure_dir(&vagga)?;
    let cmddir = vagga.join("cmd");
    safe_ensure_dir(&cmddir)?;

    for (_, ref cmd) in &ctx.config.commands {
        if let Some(link) = cmd.link() {
            let tmpname = cmddir.join(format!("{}.tmp", link.name));
            let mut f = BufWriter::new(
                File::create(&tmpname)
                .map_err(|e| format!("Can't write file {:?}: {}",
                    tmpname, e))?);
            // TODO(tailhook) properly escape command name
            // TODO(tailhook) running the script in `.vagga` is insecure
            write!(&mut f, "#!/bin/sh\n\
                    dir=$(${{VAGGA:-vagga}} _base_dir)\n\
                    exec $dir/.vagga/.cmd/{} \"$*\"\n", link.name)
                .map_err(|e| format!("Can't write file {:?}: {}",
                    tmpname, e))?;
            set_permissions(&tmpname, Permissions::from_mode(0o755))
                .map_err(|e| format!("Can't set permissions for {:?}: {}",
                    tmpname, e))?;
            let dest = cmddir.join(link.name);
            rename(&tmpname, &dest)
                .map_err(|e| format!("Can't rename file {:?} -> {:?}: {}",
                    tmpname, dest, e))?;
        }
    }

    Ok(())
}

fn update_local_scripts(ctx: &Context) -> Result<(), String> {
    let vagga = ctx.config_dir.join(".vagga");
    safe_ensure_dir(&vagga)?;
    let cmddir = vagga.join(".cmd");
    safe_ensure_dir(&cmddir)?;

    for (ref name, ref cmd) in &ctx.config.commands {
        if let Some(link) = cmd.link() {
            let tmpname = cmddir.join(format!("{}.tmp", link.name));
            let mut f = BufWriter::new(
                File::create(&tmpname)
                .map_err(|e| format!("Can't write file {:?}: {}",
                    tmpname, e))?);
            if link.path_translation {
                write!(&mut f, "#!/bin/sh\n\
                        dir=$(${{VAGGA:-vagga}} _base_dir)\n\
                        ${{VAGGA:-vagga}} {:?} \"$*\" 2>&1 \
                        | sed \"s@/work/@$dir/@g\"\n", name)
                    .map_err(|e| format!("Can't write file {:?}: {}",
                        tmpname, e))?;
            } else {
                // TODO(tailhook) properly escape command name
                write!(&mut f, "#!/bin/sh\n\
                        exec ${{VAGGA:-vagga}} {:?} \"$*\"\n", name)
                    .map_err(|e| format!("Can't write file {:?}: {}",
                        tmpname, e))?;
            }
            set_permissions(&tmpname, Permissions::from_mode(0o755))
                .map_err(|e| format!("Can't set permissions for {:?}: {}",
                    tmpname, e))?;
            let dest = cmddir.join(link.name);
            rename(&tmpname, &dest)
                .map_err(|e| format!("Can't rename file {:?} -> {:?}: {}",
                    tmpname, dest, e))?;
        }
    }

    Ok(())
}

