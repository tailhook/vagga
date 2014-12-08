use std::io::IoError;
use std::io::ALL_PERMISSIONS;
use std::io::fs::{rmdir_recursive, mkdir_recursive, mkdir};
use std::io::fs::PathExtensions;
use container::mount::bind_mount;


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
