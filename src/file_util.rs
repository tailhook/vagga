use std::io::Error;
use std::path::{Path, PathBuf};
use std::fs::read_dir;


fn read_visible_entries(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let res = vec!();
    for entry_ in try!(read_dir) {
        let entry = try!(entry_);
        if !entry.file_name().starts_with(".") {
            res.push(entry.path().to_path_buf());
        }
    }
    Ok(res)
}
