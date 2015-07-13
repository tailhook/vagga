use std::io::Error;
use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs::PermissionsExt;


fn read_visible_entries(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let res = vec!();
    for entry_ in try!(fs::read_dir(dir)) {
        let entry = try!(entry_);
        if !entry.file_name().starts_with(".") {
            res.push(entry.path().to_path_buf());
        }
    }
    Ok(res)
}

fn create_dir(path: &Path, recursive: bool) -> Result<(), Error> {
    if path.is_dir() {
        return Ok(())
    }
    if recursive {
        match path.parent() {
            Some(p) if p != path => try!(create_dir(p, true)),
            None => {}
        }
    }
    try!(fs::create_dir(path));
    try!(fs::set_permissions(path, fs::Permissions::from_mode(0o755)));
    Ok(())
}

fn create_dir_mode(path: &Path, mode: u32) -> Result<(), Error> {
    if path.is_dir() {
        return Ok(())
    }
    try!(fs::create_dir(path));
    try!(fs::set_permissions(path, fs::Permissions::from_mode(mode)));
    Ok(())
}