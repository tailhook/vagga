use time::now;
use libc::funcs::posix88::unistd::getpid;
use std::io::fs::{mkdir_recursive, mkdir, link, readdir};
use std::io::{FilePermission, TypeSymlink};
use std::io;

use super::super::env::{Environ, Container};
use super::super::config::{Command, WriteMode};
use super::super::config::{ReadOnly, TransientHardLinkCopy};
use super::super::clean::run_rmdirs;


pub fn check_command_workdir(env: &Environ, command: &Command)
    -> Result<Path, String>
{
    if command.work_dir.is_some() {
        let ncwd = env.project_root.join(
            command.work_dir.as_ref().unwrap().as_slice());
        if !env.project_root.is_ancestor_of(&ncwd) {
            return Err(format!("Command's work-dir must be relative to \
                project root"));
        }
        return Ok(ncwd);
    } else {
        return Ok(env.work_dir.clone());
    }
}

struct WriteSentinel {
    vagga_exe: Path,
    dir_to_delete: Option<Path>,
}


impl Drop for WriteSentinel {
    fn drop(&mut self) {
        match self.dir_to_delete {
            Some(ref path) => {
                run_rmdirs(&self.vagga_exe, vec!(path.clone())).unwrap();
            }
            None => {}
        }
    }
}


fn clone_dir(src: &Path, target: &Path) -> Result<(), String> {
    let items = try!(readdir(src)
        .map_err(|e| format!("Failed to read dir: {}", e)));
    for item in items.iter() {
        let stat = try!(item.lstat()
            .map_err(|e| format!("Failed to stat file: {}", e)));
        let tpath = target.join(item.path_relative_from(src).unwrap());
        match stat.kind {
            io::TypeFile | io::TypeSymlink => {
                try!(link(item, &tpath)
                    .map_err(|e| format!("Failed to hardlink file: {}", e)));
            }
            io::TypeDirectory => {
                try!(mkdir(&tpath, stat.perm)
                    .map_err(|e| format!("Failed to make directory: {}", e)));
                try!(clone_dir(item, &tpath));
            }
            io::TypeNamedPipe|io::TypeBlockSpecial|io::TypeUnknown => {
                warn!("Ignoring special file: {}", item.display());
            }
        }
    }
    return Ok(());
}


pub fn write_sentinel(env: &Environ, container: &mut Container,
    write_mode: WriteMode)
    -> Result<WriteSentinel, String>
{
    let mut res = WriteSentinel {
        vagga_exe: env.vagga_exe.clone(),
        dir_to_delete: None,
    };
    match write_mode {
        ReadOnly => {}
        TransientHardLinkCopy => {
            let time = now();
            let name = format!(
                "{}.{year:04d}{mon:02d}{mday:02d}.{}",
                container.name, unsafe { getpid() },
                year=time.tm_year, mon=time.tm_mon, mday=time.tm_mday);
            let path = env.local_vagga.join(".transients").join(name);
            try!(mkdir_recursive(&path, FilePermission::all())
                .map_err(|e| format!("Can't create container dir: {}", e)));
            res.dir_to_delete = Some(path.clone());

            try!(clone_dir(container.container_root.as_ref().unwrap(), &path));
            container.container_root = Some(path);
        }
    }
    return Ok(res);
}

pub fn is_writeable(val: WriteMode) -> bool {
    return match val {
        ReadOnly => false,
        TransientHardLinkCopy => true,
    };
}

pub fn print_banner(val: &Option<String>) {
    match *val {
        None => {}
        Some(ref x) => {
            if x.len() == 0 || x.as_slice().chars().last().unwrap() != '\n' {
                println!("{}", x);
            } else {
                print!("{}", x);
            }
        }
    }
}
