use std::io::ALL_PERMISSIONS;
use std::os::{getenv};
use std::io::BufferedReader;
use std::io::fs::{File, PathExtensions};
use std::io::fs::{mkdir};
use std::collections::TreeMap;

use config::Container;
use config::containers::{Tmpfs};
use container::mount::{bind_mount, unmount, mount_system_dirs, remount_ro};
use container::mount::{mount_tmpfs};
use container::root::change_root;
use container::uidmap::{Uidmap};
use super::run::DEFAULT_PATH;


pub fn get_environment(container: &Container)
    -> Result<TreeMap<String, String>, String>
{
    let mut result = TreeMap::new();
    result.insert("TERM".to_string(),
                  getenv("TERM").unwrap_or("dumb".to_string()));
    result.insert("PATH".to_string(),
                  DEFAULT_PATH.to_string());
    if let Some(ref filename) = container.environ_file {
        let mut f = BufferedReader::new(try!(
                File::open(filename)
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e))));
        for line_read in f.lines() {
            let line = try!(line_read
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e)));
            let mut pair = line.as_slice().splitn(2, '=');
            let key = pair.next().unwrap();
            let mut value = try!(pair.next()
                .ok_or(format!("Error reading environment file {}: bad format",
                    filename.display())));
            if value.len() > 0 && value.starts_with("\"") {
                value = value[ 1 .. value.len()-2 ];
            }
            result.insert(key.to_string(), value.to_string());
        }
    }
    for (ref k, ref v) in container.environ.iter() {
        result.insert(k.to_string(), v.to_string());
    }
    return Ok(result);
}

pub fn setup_filesystem(container: &Container, _uid_map: &Uidmap,
    container_ver: &str)
    -> Result<(), String>
{
    let root_path = Path::new("/");
    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try!(mkdir(&tgtroot, ALL_PERMISSIONS)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    try!(bind_mount(&Path::new("/vagga/roots")
                     .join(container_ver).join("root"),
                    &tgtroot)
         .map_err(|e| format!("Error bind mount: {}", e)));
    try!(remount_ro(&tgtroot));
    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));
    for (path, vol) in container.volumes.iter() {
        match vol {
            &Tmpfs(params) => {
                try!(mount_tmpfs(&tgtroot
                    .join(path.path_relative_from(&root_path).unwrap()),
                    format!("size={:u},mode=0{:o}", params.size, params.mode)
                    .as_slice()));
            }
        }
    }
    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting old root: {}", e)));
    Ok(())
}
