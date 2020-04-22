use std::env;
use std::fs::read_link;
use std::io::{stdout, stderr};
use std::path::Path;
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser};

use config::volumes::{Volume, PersistentInfo};
use config::command::{CommandInfo, WriteMode, Run};

use super::setup;
use super::Wrapper;
use super::util::{gen_command, warn_if_data_container};
use process_util::{run_and_wait, convert_status, copy_env_vars};
use process_util::{set_fake_uidmap};
use wrapper::init_persistent::{Guard, PersistentVolumeGuard};


pub fn commandline_cmd(cmd_name: &str, command: &CommandInfo,
    wrapper: &Wrapper, mut cmdline: Vec<String>)
    -> Result<i32, String>
{
    // TODO(tailhook) detect other shells too
    let has_args = command.accepts_arguments
            .unwrap_or(!matches!(command.run, Run::Shell(..)));
    let mut args = Vec::new();
    if !has_args {
        let mut ap = ArgumentParser::new();
        ap.set_description(command.description.as_ref()
            .map(|x| &x[..]).unwrap_or(""));
        ap.stop_on_first_argument(true);
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    } else {
        cmdline.remove(0);
        args.extend(cmdline.into_iter());
    }

    let pid: pid_t = read_link(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.to_str().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("Can't parse pid: {:?}", v)))?;
    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;

    let cconfig = wrapper.config.containers.get(&command.container)
        .ok_or(format!("Container {} not found", command.container))?;

    let write_mode = match command.write_mode {
        WriteMode::read_only => setup::WriteMode::ReadOnly,
        WriteMode::transient_hard_link_copy
        => setup::WriteMode::TransientHardlinkCopy(pid),
    };
    let cont_ver = wrapper.root.as_ref().unwrap();
    let mut setup_info = setup::SetupInfo::from_container(&cconfig);
    setup_info
        .volumes(&command.volumes)
        .write_mode(write_mode);
    let mut guards = Vec::<Box<dyn Guard>>::new();
    for (_, &volume) in &setup_info.volumes {
        match volume {
            &Volume::Persistent(ref info @ PersistentInfo {
                init_command: Some(_), .. })
            if info.init_command.as_ref().unwrap() == cmd_name => {
                match PersistentVolumeGuard::new(&info) {
                    Ok(Some(guard)) => {
                        guards.push(Box::new(guard));
                        setup_info.tmp_volumes.insert(&info.name);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        return Err(format!("Persistent volume {:?} error: {}",
                                           info.name, e));
                    }
                }
            }
            _ => {}
        }
    }
    warn_if_data_container(&cconfig);

    setup::setup_filesystem(&setup_info, &cont_ver)?;

    let env = setup::get_environment(&wrapper.settings, cconfig,
        Some(&command))?;
    let mut cmd = gen_command(&cconfig.default_shell, &command.run, &env)?;
    cmd.args(&args);
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    if let Some(euid) = command.external_user_id {
        set_fake_uidmap(&mut cmd, command.user_id, euid)?;
    }
    cmd.uid(command.user_id);
    cmd.gid(command.group_id);
    cmd.groups(command.supplementary_gids.clone());
    if let Some(ref wd) = command.work_dir {
        cmd.current_dir(Path::new("/work").join(&wd));
    } else {
        cmd.current_dir(env::var("_VAGGA_WORKDIR")
                        .unwrap_or("/work".to_string()));
    }
    for (ref k, ref v) in env.iter() {
        cmd.env(k, v);
    }

    let result = run_and_wait(&mut cmd)
                .map(convert_status);
    if result == Ok(0) {
        for guard in guards {
            guard.commit()
                .map_err(|e| format!("Error commiting guard: {}", e))?;
        }
    }
    return result;
}
