use std::io::Write;
use std::fs::{File, set_permissions, Permissions};
use std::path::{PathBuf, Path};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;

use path_util::ToRelative;
use builder::guard::Guard;
use builder::error::StepError;


pub fn write_text_files(files: &BTreeMap<PathBuf, String>, _guard: &mut Guard)
    -> Result<(), StepError>
{
    for (path, text) in files.iter() {
        let realpath = Path::new("/vagga/root")
            .join(path.rel());
        try!(File::create(&realpath)
            .and_then(|mut f| f.write_all(text.as_bytes()))
            .map_err(|e| format!("Can't create file: {}", e)));
        try!(set_permissions(&realpath,
            Permissions::from_mode(0o644))
            .map_err(|e| format!("Can't chmod file: {}", e)));
    }
    Ok(())
}
