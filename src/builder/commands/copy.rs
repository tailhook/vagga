use std::fs::{symlink_metadata};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

use regex::Regex;
use scan_dir::{ScanDir};

use file_util::{create_dir_mode, shallow_copy};
use path_util::ToRelative;
use config::builders::Copy;
use builder::guard::Guard;
use builder::error::StepError;
use builder::error::StepError as E;


pub fn copy(cinfo: &Copy, _guard: &mut Guard)
    -> Result<(), StepError>
{
    let ref src = cinfo.source;
    let dest = Path::new("/vagga/root").join(cinfo.path.rel());
    let typ = try!(symlink_metadata(src)
        .map_err(|e| E::Write(src.into(), e)));
    if typ.is_dir() {
        try!(create_dir_mode(&dest, typ.permissions().mode())
            .map_err(|e| E::Write(dest.clone(), e)));
        let re = try!(Regex::new(&cinfo.ignore_regex)
            .map_err(|e| E::Regex(Box::new(e))));
        try!(ScanDir::all().walk(src, |iter| {
            for (entry, _) in iter {
                let fpath = entry.path();
                // We know that directory is inside
                // the source
                let path = fpath.rel_to(src).unwrap();
                // We know that it's decodable
                let strpath = path.to_str().unwrap();
                if re.is_match(strpath) {
                    continue;
                }
                let fdest = dest.join(path);
                try!(shallow_copy(&fpath, &fdest,
                        cinfo.owner_uid, cinfo.owner_gid)
                    .map_err(|e| E::Write(fdest, e)));
            }
            Ok(())
        }).map_err(E::ScanDir).and_then(|x| x));
    } else {
        try!(shallow_copy(&cinfo.source, &dest,
                          cinfo.owner_uid, cinfo.owner_gid)
             .map_err(|e| E::Write(dest.clone(), e)));
    }
    Ok(())
}
