use std::io::ALL_PERMISSIONS;
use std::io::fs::{rmdir_recursive, mkdir_recursive, mkdir, rename};
use std::io::fs::PathExtensions;
use container::mount::{bind_mount};


pub fn prepare_tmp_root_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        try!(rmdir_recursive(path)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    try!(mkdir_recursive(path, ALL_PERMISSIONS)
         .map_err(|x| format!("Error creating directory: {}", x)));
    let rootdir = path.join("root");
    try!(mkdir(&rootdir, ALL_PERMISSIONS)
         .map_err(|x| format!("Error creating directory: {}", x)));
    let tgtroot = Path::new("/vagga/root");
    try!(mkdir(&tgtroot, ALL_PERMISSIONS)
         .map_err(|x| format!("Error creating directory: {}", x)));
    try!(bind_mount(&rootdir, &tgtroot));
    return Ok(());
}

pub fn commit_root(tmp_path: &Path, final_path: &Path) -> Result<(), String> {
    let mut path_to_remove = None;
    if final_path.exists() {
        let rempath = tmp_path.with_filename(
            tmp_path.filename_str().unwrap().to_string() + ".old");
        try!(rename(final_path, &rempath)
             .map_err(|x| format!("Error renaming old dir: {}", x)));
        path_to_remove = Some(rempath);
    }
    try!(rename(tmp_path, final_path)
         .map_err(|x| format!("Error renaming dir: {}", x)));
    if let Some(ref path_to_remove) = path_to_remove {
        try!(rmdir_recursive(path_to_remove)
             .map_err(|x| format!("Error removing old dir: {}", x)));
    }
    return Ok(());
}

