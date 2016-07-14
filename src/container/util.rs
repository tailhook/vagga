use std::io;
use std::fs::{read_dir, remove_file, remove_dir};
use std::fs::{symlink_metadata, read_link, hard_link};
use std::os::unix::fs::{symlink, MetadataExt};
use std::path::{Path, PathBuf};

use libc::{uid_t, gid_t};

use super::root::temporary_change_root;
use file_util::{create_dir_mode, shallow_copy, set_owner_group};

quick_error!{
    #[derive(Debug)]
    pub enum CopyDirError {
        ReadDir(path: PathBuf, err: io::Error) {
            display("Can't read dir {:?}: {}", path, err)
        }
        Stat(path: PathBuf, err: io::Error) {
            display("Can't stat {:?}: {}", path, err)
        }
        CopyFile(src: PathBuf, dst: PathBuf, err: io::Error) {
            display("Can't copy {:?} -> {:?}: {}", src, dst, err)
        }
        CreateDir(path: PathBuf, err: io::Error) {
            display("Can't create dir {:?}: {}", path, err)
        }
        ReadLink(path: PathBuf, err: io::Error) {
            display("Can't read symlink {:?}: {}", path, err)
        }
        Symlink(path: PathBuf, err: io::Error) {
            display("Can't create symlink {:?}: {}", path, err)
        }
    }
}

pub fn clean_dir<P: AsRef<Path>>(dir: P, remove_dir_itself: bool) -> Result<(), String> {
    _clean_dir(dir.as_ref(), remove_dir_itself)
}

fn _clean_dir(dir: &Path, remove_dir_itself: bool) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    // We temporarily change root, so that symlinks inside the dir
    // would do no harm. But note that dir itself can be a symlink
    try!(temporary_change_root::<_, _, _, String>(dir, || {
        let mut path = PathBuf::from("/");
        let diriter = try_msg!(read_dir(&path),
             "Can't read directory {d:?}: {err}", d=dir);
        let mut stack = vec![diriter];
        'next_dir: while let Some(mut diriter) = stack.pop() {
            while let Some(entry) = diriter.next() {
                let entry = try_msg!(entry, "Error reading dir entry: {err}");
                let typ = try_msg!(entry.file_type(),
                    "Can't stat {p:?}: {err}", p=entry.path());
                path.push(entry.file_name());
                if typ.is_dir() {
                    stack.push(diriter);  // push directory back to stack
                    let niter = try!(read_dir(&path)
                         .map_err(|e| format!("Can't read directory {:?}: {}",
                                              dir, e)));
                    stack.push(niter);  // push new directory to stack
                    continue 'next_dir;
                } else {
                    try_msg!(remove_file(&path),
                        "Can't remove file {dir:?}: {err}", dir=entry.path());
                    path.pop();
                }
            }
            if Path::new(&path) == Path::new("/") {
                break;
            } else {
                try_msg!(remove_dir(&path),
                    "Can't remove dir {p:?}: {err}", p=path);
                path.pop();
            }
        }
        Ok(())
    }));
    if remove_dir_itself {
        try_msg!(remove_dir(dir),
            "Can't remove dir {dir:?}: {err}", dir=dir);
    }
    return Ok(());
}

pub fn copy_dir(old: &Path, new: &Path,
    owner_uid: Option<uid_t>, owner_gid: Option<gid_t>)
    -> Result<(), CopyDirError>
{
    use self::CopyDirError::*;
    // TODO(tailhook) use reflinks if supported
    let dir = try!(read_dir(old).map_err(|e| ReadDir(old.to_path_buf(), e)));
    let mut stack = vec![dir];
    let mut oldp = old.to_path_buf();
    let mut newp = new.to_path_buf();
    'next_dir: while let Some(mut dir) = stack.pop() {
        while let Some(item) = dir.next() {
            let entry = try!(item.map_err(|e| ReadDir(old.to_path_buf(), e)));
            let filename = entry.file_name();
            oldp.push(&filename);
            newp.push(&filename);

            let copy_rc = try!(shallow_copy(&oldp, &newp, owner_uid, owner_gid, None)
                .map_err(|e| CopyFile(oldp.clone(), newp.clone(), e)));
            if !copy_rc {
                stack.push(dir);  // Return dir to stack
                let ndir = try!(read_dir(&oldp)
                    .map_err(|e| ReadDir(oldp.to_path_buf(), e)));
                stack.push(ndir); // Add new dir to the stack too
                continue 'next_dir;
            }
            oldp.pop();
            newp.pop();
        }
        oldp.pop();
        newp.pop();
    }
    Ok(())
}

pub fn hardlink_dir(old: &Path, new: &Path) -> Result<(), CopyDirError> {
    use self::CopyDirError::*;
    // TODO(tailhook) use reflinks if supported
    let dir = try!(read_dir(old).map_err(|e| ReadDir(old.to_path_buf(), e)));
    let mut stack = vec![dir];
    let mut oldp = old.to_path_buf();
    let mut newp = new.to_path_buf();
    'next_dir: while let Some(mut dir) = stack.pop() {
        while let Some(item) = dir.next() {
            let entry = try!(item.map_err(|e| ReadDir(old.to_path_buf(), e)));
            let filename = entry.file_name();
            oldp.push(&filename);
            newp.push(&filename);

            let typ = try!(entry.file_type()
                .map_err(|e| Stat(oldp.clone(), e)));
            if typ.is_file() {
                try!(hard_link(&oldp, &newp)
                    .map_err(|e| CopyFile(oldp.clone(), newp.clone(), e)));
            } else if typ.is_dir() {
                let stat = try!(symlink_metadata(&oldp)
                    .map_err(|e| Stat(oldp.clone(), e)));
                if !newp.is_dir() {
                    try!(create_dir_mode(&newp, stat.mode())
                        .map_err(|e| CreateDir(newp.clone(), e)));
                    try!(set_owner_group(&newp, stat.uid(), stat.gid())
                        .map_err(|e| CreateDir(newp.clone(), e)));
                }
                stack.push(dir);  // Return dir to stack
                let ndir = try!(read_dir(&oldp)
                    .map_err(|e| ReadDir(oldp.to_path_buf(), e)));
                stack.push(ndir); // Add new dir to the stack too
                continue 'next_dir;
            } else if typ.is_symlink() {
                let lnk = try!(read_link(&oldp)
                               .map_err(|e| ReadLink(oldp.clone(), e)));
                try!(symlink(&lnk, &newp)
                    .map_err(|e| Symlink(newp.clone(), e)));
            } else {
                warn!("Unknown file type {:?}", &entry.path());
            }
            oldp.pop();
            newp.pop();
        }
        oldp.pop();
        newp.pop();
    }
    Ok(())
}

pub fn version_from_symlink<P: AsRef<Path>>(path: P) -> Result<String, String>
{
    let lnk = path.as_ref();
    let path = try!(read_link(&path)
        .map_err(|e| format!("Can't read link {:?}: {}", lnk, e)));
    path.iter().rev().nth(1).and_then(|x| x.to_str())
    .ok_or_else(|| format!("Bad symlink {:?}: {:?}", lnk, path))
    .map(|x| x.to_string())
}
