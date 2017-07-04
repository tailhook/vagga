use std::collections::{HashMap, HashSet};
use std::fs::{File, Metadata};
use std::fs::{read_dir, remove_file, remove_dir, rename};
use std::fs::{symlink_metadata, read_link, hard_link};
use std::io::{self, BufReader, BufWriter, Seek, SeekFrom};
use std::os::unix::fs::{symlink, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use dir_signature::{self, v1, ScannerConfig as Sig};
use dir_signature::HashType;
use dir_signature::v1::{Entry, EntryKind, Hashes, Parser, ParseError};
use dir_signature::v1::merge::FileMergeBuilder;
use libc::{uid_t, gid_t};
use tempfile::tempfile;
use quick_error::ResultExt;

use super::root::temporary_change_root;
use file_util::{Dir, ShallowCopy};

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
    temporary_change_root::<_, _, _, String>(dir, || {
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
                    let niter = read_dir(&path)
                         .map_err(|e| format!("Can't read directory {:?}: {}",
                                              dir, e))?;
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
    })?;
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
    let dir = read_dir(old).map_err(|e| ReadDir(old.to_path_buf(), e))?;
    let mut stack = vec![dir];
    let mut oldp = old.to_path_buf();
    let mut newp = new.to_path_buf();
    'next_dir: while let Some(mut dir) = stack.pop() {
        while let Some(item) = dir.next() {
            let entry = item.map_err(|e| ReadDir(old.to_path_buf(), e))?;
            let filename = entry.file_name();
            oldp.push(&filename);
            newp.push(&filename);

            let copy_rc = ShallowCopy::new(&oldp, &newp)
                .owner_uid(owner_uid)
                .owner_gid(owner_gid)
                .copy()
                .map_err(|e| CopyFile(oldp.clone(), newp.clone(), e))?;
            if !copy_rc {
                stack.push(dir);  // Return dir to stack
                let ndir = read_dir(&oldp)
                    .map_err(|e| ReadDir(oldp.to_path_buf(), e))?;
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
    let dir = read_dir(old).map_err(|e| ReadDir(old.to_path_buf(), e))?;
    let mut stack = vec![dir];
    let mut oldp = old.to_path_buf();
    let mut newp = new.to_path_buf();
    'next_dir: while let Some(mut dir) = stack.pop() {
        while let Some(item) = dir.next() {
            let entry = item.map_err(|e| ReadDir(old.to_path_buf(), e))?;
            let filename = entry.file_name();
            oldp.push(&filename);
            newp.push(&filename);

            let typ = entry.file_type()
                .map_err(|e| Stat(oldp.clone(), e))?;
            if typ.is_file() {
                hard_link(&oldp, &newp)
                    .map_err(|e| CopyFile(oldp.clone(), newp.clone(), e))?;
            } else if typ.is_dir() {
                let stat = symlink_metadata(&oldp)
                    .map_err(|e| Stat(oldp.clone(), e))?;
                if !newp.is_dir() {
                    Dir::new(&newp)
                            .mode(stat.mode())
                            .uid(stat.uid())
                            .gid(stat.gid())
                            .create()
                        .map_err(|e| CreateDir(newp.clone(), e))?;
                }
                stack.push(dir);  // Return dir to stack
                let ndir = read_dir(&oldp)
                    .map_err(|e| ReadDir(oldp.to_path_buf(), e))?;
                stack.push(ndir); // Add new dir to the stack too
                continue 'next_dir;
            } else if typ.is_symlink() {
                let lnk = read_link(&oldp)
                               .map_err(|e| ReadLink(oldp.clone(), e))?;
                symlink(&lnk, &newp)
                    .map_err(|e| Symlink(newp.clone(), e))?;
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
    let path = read_link(&path)
        .map_err(|e| format!("Can't read link {:?}: {}", lnk, e))?;
    path.iter().rev().nth(1).and_then(|x| x.to_str())
    .ok_or_else(|| format!("Bad symlink {:?}: {:?}", lnk, path))
    .map(|x| x.to_string())
}

pub fn write_container_signature(cont_dir: &Path)
    -> Result<(), String>
{
    let index = File::create(cont_dir.join("index.ds1"))
        .map_err(|e| format!("Can't write index: {}", e))?;
    v1::scan(Sig::new()
            .auto_threads()
            .hash(HashType::blake2b_256())
            .add_dir(cont_dir.join("root"), "/"),
        &mut BufWriter::new(index)
    ).map_err(|e| format!("Error indexing: {}", e))
}

#[derive(Debug)]
pub struct Diff {
    pub missing_paths: Vec<PathBuf>,
    pub extra_paths: Vec<PathBuf>,
    pub corrupted_paths: Vec<PathBuf>,
}

quick_error!{
    #[derive(Debug)]
    pub enum CheckSignatureError {
        NoSignatureFile(path: PathBuf, err: io::Error) {
            description("missing signature file")
            display("Missing signature file {:?}: {}", path, err)
        }
        ReadSignatureFile(path: PathBuf, err: io::Error) {
            description("error reading signature file")
            display("Error reading signature file {:?}: {}", path, err)
        }
        ReadTempSignatureFile(err: io::Error) {
            description("error reading temporary signature file")
            display("Error reading temporary signature file: {}", err)
        }
        CreateTempSignatureFile(err: io::Error) {
            description("error creating temporary signature file")
            display("Error creating temporary signature file: {}", err)
        }
        Scan(err: dir_signature::Error) {
            description("error indexing container")
            display("Error indexing container: {}", err)
            from()
        }
        ParseSignature(err: ParseError) {
            description("error parsing signature file")
            display("Error parsing signature file: {}", err)
            from()
        }
    }
}

#[cfg(feature="containers")]
pub fn check_signature(cont_dir: &Path)
    -> Result<Option<Diff>, CheckSignatureError>
{
    use self::CheckSignatureError::*;

    let ds_path = cont_dir.join("index.ds1");
    let mut ds_file = File::open(&ds_path)
        .map_err(|e| NoSignatureFile(ds_path.clone(), e))?;
    let ds_hash = dir_signature::get_hash(&mut ds_file)
        .map_err(|e| ReadSignatureFile(ds_path.clone(), e))?;

    let mut scanner_config = Sig::new();
    scanner_config
        .auto_threads()
        .hash(HashType::blake2b_256())
        .add_dir(cont_dir.join("root"), "/");
    let mut real_ds_file = tempfile()
        .map_err(|e| CreateTempSignatureFile(e))?;
    v1::scan(&scanner_config, &mut real_ds_file)
        .map_err(|e| Scan(e))?;
    real_ds_file.seek(SeekFrom::Start(0))
        .map_err(|e| ReadTempSignatureFile(e))?;
    let real_ds_hash = dir_signature::get_hash(&mut real_ds_file)
        .map_err(|e| ReadTempSignatureFile(e))?;

    if ds_hash != real_ds_hash {
        let mut ds_reader = BufReader::new(ds_file);
        let mut real_ds_reader = BufReader::new(real_ds_file);
        ds_reader.seek(SeekFrom::Start(0))
            .map_err(|e| ReadSignatureFile(ds_path.clone(), e))?;
        real_ds_reader.seek(SeekFrom::Start(0))
            .map_err(|e| ReadTempSignatureFile(e))?;
        let mut ds_parser = Parser::new(ds_reader)?;
        let mut real_ds_parser = Parser::new(real_ds_reader)?;

        let mut diff = Diff {
            missing_paths: vec!(),
            extra_paths: vec!(),
            corrupted_paths: vec!(),
        };
        {
            let mut real_ds_iter = real_ds_parser.iter();
            for entry in ds_parser.iter() {
                let entry = entry?;
                match real_ds_iter.advance(&entry.kind()) {
                    Some(Ok(real_entry)) => {
                        if entry != real_entry {
                            diff.corrupted_paths.push(
                                entry.path().to_path_buf());
                        }
                    },
                    Some(Err(e)) => {
                        return Err(CheckSignatureError::from(e));
                    },
                    None => {
                        diff.missing_paths.push(entry.path().to_path_buf());
                    },
                }
            }
        }

        let mut ds_reader = ds_parser.into_reader();
        let mut real_ds_reader = real_ds_parser.into_reader();
        ds_reader.seek(SeekFrom::Start(0))
            .map_err(|e| ReadSignatureFile(ds_path.clone(), e))?;
        real_ds_reader.seek(SeekFrom::Start(0))
            .map_err(|e| ReadTempSignatureFile(e))?;
        let mut ds_parser = Parser::new(ds_reader)?;
        let mut real_ds_parser = Parser::new(real_ds_reader)?;

        let mut ds_iter = ds_parser.iter();
        for real_entry in real_ds_parser.iter() {
            let real_entry = real_entry?;
            match ds_iter.advance(&real_entry.kind()) {
                Some(Err(e)) => {
                    return Err(CheckSignatureError::from(e));
                },
                None => {
                    diff.extra_paths.push(real_entry.path().to_path_buf());
                },
                _ => {},
            }
        }

        Ok(Some(diff))
    } else {
        Ok(None)
    }
}

#[cfg(not(feature="containers"))]
pub fn check_signature(cont_dir: &Path)
    -> Result<Option<Diff>, CheckSignatureError>
{
    unimplemented!();
}

quick_error!{
    #[derive(Debug)]
    pub enum HardlinkError {
        OpenSignatureFile(path: PathBuf, err: io::Error) {
            description("error opening file")
            display("Error opening signature file {:?}: {}", path, err)
        }
        ParseSignature(path: PathBuf, err: ParseError) {
            description("error parsing signature file")
            display("Error parsing signature file {:?}: {}", path, err)
            context(path: &'a Path, err: ParseError)
                -> (path.to_path_buf(), err)
        }
        MergedSignatures(err: v1::merge::MergeError) {
            description("error merging signature files")
            display("Error merging signature files")
            from()
        }
        InvalidEntry(sig_path: PathBuf, entry_path: PathBuf) {
            description("invalid signature entry")
            display("Invalid signature entry in file {:?}: {:?}",
                sig_path, entry_path)
        }
        RemoveTempFile(path: PathBuf, err: io::Error) {
            description("error removing temporary file")
            display("Error removing temporary file {:?}: {}", path, err)
        }
        StatFile(path: PathBuf, err: io::Error) {
            description("error querying file stats")
            display("Error querying file stats {:?}: {}", path, err)
        }
        OpenFile(path: PathBuf, err: io::Error) {
            description("error opeining file")
            display("Error opeining file {:?}: {}", path, err)
        }
        HashFile(path: PathBuf, err: io::Error) {
            description("error hashing file")
            display("Error hashing file {:?}: {}", path, err)
            context(path: &'a Path, err: io::Error)
                -> (path.to_path_buf(), err)
        }
        LinkFile(tgt: PathBuf, lnk: PathBuf, err: LinkError) {
            description("error hard linking file")
            display("Error hard linking {:?} -> {:?}: {}", tgt, lnk, err)
        }
    }
}

#[cfg(feature="containers")]
pub fn hardlink_container_files<I, P>(tmp_dir: &Path, cont_dirs: I)
    -> Result<(u32, u64), HardlinkError>
    where I: IntoIterator<Item = P>, P: AsRef<Path>
{
    use self::HardlinkError::*;

    let container_root = tmp_dir.join("root");
    let main_ds_path = tmp_dir.join("index.ds1");
    if !main_ds_path.exists() {
        warn!("No index file exists, can't hardlink container");
        return Ok((0, 0));
    }
    let main_ds_reader = BufReader::new(File::open(&main_ds_path)
        .map_err(|e| OpenSignatureFile(main_ds_path.clone(), e))?);
    let mut main_ds_parser = Parser::new(main_ds_reader)
        .context(main_ds_path.as_path())?;

    let mut merged_ds_builder = FileMergeBuilder::new();
    for cont_path in cont_dirs {
        let cont_path = cont_path.as_ref();
        info!("Found container to hardlink with: {:?}", cont_path);
        merged_ds_builder.add(&cont_path.join("root"),
                              &cont_path.join("index.ds1"));
    }
    let mut merged_ds = merged_ds_builder.finalize()?;
    let mut merged_ds_iter = merged_ds.iter();

    let tmp = tmp_dir.join(".link.tmp");
    if tmp.exists() {
        remove_file(&tmp).map_err(|e| RemoveTempFile(tmp.clone(), e))?;
    }
    let mut count = 0;
    let mut size = 0;
    for entry in main_ds_parser.iter() {
        match entry {
            Ok(Entry::File{
                path: ref lnk_path,
                exe: lnk_exe,
                size: lnk_size,
                hashes: ref lnk_hashes,
            }) => {
                let lnk = container_root.join(
                    lnk_path.strip_prefix("/").map_err(|_|
                        InvalidEntry(main_ds_path.clone(), lnk_path.clone()))?);
                let lnk_stat = lnk.symlink_metadata()
                    .map_err(|e| StatFile(lnk.clone(), e))?;
                for tgt_entry in merged_ds_iter
                    .advance(&EntryKind::File(lnk_path))
                {
                    match tgt_entry {
                        (tgt_root,
                         Ok(Entry::File{
                             path: ref tgt_path,
                             exe: tgt_exe,
                             size: tgt_size,
                             hashes: ref tgt_hashes}))
                            if lnk_exe == tgt_exe &&
                            lnk_size == tgt_size &&
                            lnk_hashes == tgt_hashes =>
                        {
                            let ref tgt = tgt_root.join(
                                tgt_path.strip_prefix("/").map_err(|_|
                                    InvalidEntry(
                                        tgt_root.with_file_name("index.ds1"),
                                        tgt_path.clone()))?);
                            if maybe_link_file(
                                tgt, &lnk, &lnk_stat, &lnk_hashes, &tmp)?
                            {
                                trace!("Hardlinking {:?} -> {:?}", lnk, tgt);
                                count += 1;
                                size += tgt_size;
                                break;
                            }
                        },
                        (_, Ok(_)) => continue,
                        (tgt_root, Err(e)) => {
                            return Err(ParseSignature(
                                tgt_root.with_file_name("index.ds1"), e));
                        }
                    }
                }
            },
            Ok(_) => {},
            Err(e) => return Err(ParseSignature(main_ds_path.clone(), e)),
        }
    }

    Ok((count, size))
}

#[cfg(not(feature="containers"))]
pub fn hardlink_container_files<I, P>(tmp_dir: &Path, cont_dirs: I)
    -> Result<(u32, u64), String>
    where I: IntoIterator<Item = P>, P: AsRef<Path>
{
    unimplemented!();
}

#[cfg(feature="containers")]
fn maybe_link_file(tgt: &Path,
    lnk: &Path, lnk_stat: &Metadata, lnk_hashes: &Hashes, tmp: &Path)
    -> Result<bool, HardlinkError>
{
    use self::HardlinkError::*;

    let tgt_stat = match tgt.symlink_metadata() {
        Ok(s) => s,
        Err(ref e)
            if e.kind() == io::ErrorKind::NotFound =>
        {
            // Ignore not found error cause container
            // could be deleted
            return Ok(false);
        },
        Err(e) => {
            return Err(StatFile(tgt.to_path_buf(), e));
        },
    };
    if lnk_stat.mode() != tgt_stat.mode() ||
        lnk_stat.uid() != tgt_stat.uid() ||
        lnk_stat.gid() != lnk_stat.gid()
    {
        return Ok(false);
    }
    let mut tgt_file = match File::open(tgt) {
        Ok(f) => f,
        Err(ref e)
            if e.kind() == io::ErrorKind::NotFound =>
        {
            return Ok(false);
        },
        Err(e) => return Err(OpenFile(tgt.to_path_buf(), e)),
    };
    if !lnk_hashes.check_file(&mut tgt_file)
        .map_err(|e| HashFile(tgt.to_path_buf(), e))?
    {
        warn!("Mismatch file hash: {:?}", tgt);
        return Ok(false);
    }
    match safe_hardlink(tgt, &lnk, &tmp) {
        Ok(_) => {
            Ok(true)
        },
        Err(LinkError::Create(ref e))
            if e.kind() == io::ErrorKind::NotFound =>
        {
            // Ignore not found error cause container could be deleted
            Ok(false)
        },
        Err(e) => {
            Err(LinkFile(tgt.to_path_buf(), lnk.to_path_buf(), e))
        },
    }
}

#[cfg(not(feature="containers"))]
fn maybe_link_file(tgt: &Path,
    lnk: &Path, lnk_stat: &Metadata, lnk_hashes: &Hashes, tmp: &Path)
    -> Result<bool, HardlinkError>
{
    unimplemented!()
}

#[cfg(feature="containers")]
pub fn hardlink_all_identical_files<I, P>(cont_dirs: I)
    -> Result<(u64, u64), HardlinkError>
    where I: IntoIterator<Item = P>, P: AsRef<Path>
{
    use self::HardlinkError::*;

    let mut merged_ds_builder = FileMergeBuilder::new();
    for cont_dir in cont_dirs {
        let cont_dir = cont_dir.as_ref();
        info!("Found container for hardlinking: {:?}", cont_dir);
        merged_ds_builder.add(cont_dir, &cont_dir.join("index.ds1"));
    }
    let mut merged_ds = merged_ds_builder.finalize()?;
    let merged_ds_iter = merged_ds.iter();

    let mut count = 0;
    let mut size = 0;
    let mut grouped_entries = HashMap::new();
    let mut linked_inodes = HashSet::new();
    'outer:
    for cont_dirs_and_entries in merged_ds_iter {
        grouped_entries.clear();

        for (cont_dir, entry) in cont_dirs_and_entries.into_iter() {
            match entry {
                Ok(Entry::File{path, exe, size, hashes}) => {
                    let full_path = cont_dir.join("root").join(
                        path.strip_prefix("/").map_err(|_| InvalidEntry(
                            cont_dir.join("index.ds1"), path.clone()))?);
                    let meta = match full_path.symlink_metadata() {
                        Ok(meta) => meta,
                        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                            continue;
                        },
                        Err(e) => {
                            return Err(StatFile(full_path.clone(), e));
                        },
                    };
                    grouped_entries.entry(
                            EntryKey {
                                exe: exe,
                                size: size,
                                hashes: hashes,
                                mode: meta.mode(),
                                uid: meta.uid(),
                                gid: meta.gid(),
                            })
                        .or_insert(vec!())
                        .push((cont_dir, full_path, meta));
                },
                Ok(_) => continue 'outer,
                Err(e) => {
                    return Err(ParseSignature(cont_dir.join("index.ds1"), e));
                },
            }
        }

        for (&EntryKey{ref hashes, ..}, paths_and_metas) in &grouped_entries {
            if let Some((&(_, ref tgt_path, ref tgt_meta), links)) =
                paths_and_metas.split_first()
            {
                let mut tgt_file = match File::open(tgt_path) {
                    Ok(f) => f,
                    Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                        continue;
                    },
                    Err(e) => return Err(OpenFile(tgt_path.clone(), e)),
                };
                if !hashes.check_file(&mut tgt_file)
                    .map_err(|e| HashFile(tgt_path.clone(), e))?
                {
                    warn!("Mismatch file hash: {:?}", tgt_path);
                    break;
                }
                let tgt_ino = tgt_meta.ino();
                linked_inodes.clear();
                for &(ref cont_dir, ref lnk_path, ref lnk_meta) in links {
                    let tmp_path = cont_dir.join(".lnk.tmp");
                    let lnk_ino = lnk_meta.ino();
                    if lnk_ino == tgt_ino {
                        continue;
                    }
                    match safe_hardlink(tgt_path, lnk_path, &tmp_path) {
                        Ok(_) => {
                            if !linked_inodes.contains(&lnk_ino) {
                                count += 1;
                                size += lnk_meta.size();
                            }
                            linked_inodes.insert(lnk_ino);
                        },
                        Err(LinkError::Create(ref e)) |
                        Err(LinkError::Rename(ref e))
                            if e.kind() == io::ErrorKind::NotFound =>
                        {
                            // Ignore not found error cause container
                            // could be deleted
                            continue;
                        },
                        Err(e) => {
                            return Err(LinkFile(
                                tgt_path.clone(), lnk_path.clone(), e));
                        },
                    }
                }
            }
        }
    }

    Ok((count, size))
}

#[cfg(not(feature="containers"))]
pub fn hardlink_all_identical_files<I, P>(cont_dirs: I)
    -> Result<(u64, u64), String>
    where I: IntoIterator<Item = P>, P: AsRef<Path>
{
    unimplemented!();
}

#[derive(PartialEq, Eq, Hash)]
struct EntryKey {
    pub exe: bool,
    pub size: u64,
    pub hashes: Hashes,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
}

pub fn collect_containers_from_storage(storage_dir: &Path)
    -> Result<Vec<ContainerDir>, String>
{
    let mut cont_dirs = vec!();
    for entry in try_msg!(read_dir(storage_dir),
        "Error reading storage directory: {err}")
    {
        match entry {
            Ok(entry) => {
                let project_dir = entry.path();
                if !project_dir.is_dir() {
                    continue;
                }
                let project_name =
                    if let Some(project_name) = project_dir.file_name()
                    .and_then(|n| n.to_str())
                {
                    project_name
                } else {
                    continue;
                };
                if project_name.starts_with('.') {
                    continue;
                }
                let roots = project_dir.join(".roots");
                if !roots.is_dir() {
                    continue;
                }
                cont_dirs.append(
                    &mut collect_container_dirs(&roots, Some(project_name))?);
            },
            Err(e) => {
                return Err(format!("Error iterating directory: {}", e));
            },
        }
    }
    Ok(cont_dirs)
}

pub fn collect_container_dirs(roots: &Path, project_name: Option<&str>)
    -> Result<Vec<ContainerDir>, String>
{
    let mut cont_dirs = vec!();
    for entry in try_msg!(read_dir(&roots),
        "Error reading directory {path:?}: {err}", path=&roots)
    {
        match entry {
            Ok(entry) => {
                let root_dir = entry.path();
                if !root_dir.is_dir() {
                    continue;
                }
                let dir_name = if let Some(dir_name) = root_dir.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_string())
                {
                    dir_name
                } else {
                    continue;
                };
                if dir_name.starts_with(".") {
                    continue;
                }
                let container_name = {
                    let mut dir_name_parts = dir_name.rsplitn(2, '.');
                    dir_name_parts.next();
                    if let Some(cont_name) = dir_name_parts.next()
                        .map(|n| n.to_string())
                    {
                        cont_name
                    } else {
                        continue;
                    }
                };
                let date_modified = if let Ok(dt) = root_dir.symlink_metadata()
                    .and_then(|m| m.modified())
                {
                    dt
                } else {
                    continue;
                };
                cont_dirs.push(ContainerDir {
                    path: root_dir,
                    name: container_name,
                    modified: date_modified,
                    project: project_name.map(|n| n.to_string()),
                });
            },
            Err(e) => {
                return Err(format!("Error iterating directory: {}", e));
            },
        }
    }
    Ok(cont_dirs)
}

#[derive(Debug)]
pub struct ContainerDir {
    pub path: PathBuf,
    pub name: String,
    pub modified: SystemTime,
    pub project: Option<String>,
}

quick_error!{
    #[derive(Debug)]
    pub enum LinkError {
        Create(err: io::Error) {
            description("error creating hard link")
            display("Error creating hard link: {}", err)
        }
        Rename(err: io::Error) {
            description("error renaming file")
            display("Error renaming file: {}", err)
        }
        Remove(err: io::Error) {
            description("error removing file")
            display("Error removing file: {}", err)
        }
    }
}

fn safe_hardlink(tgt: &Path, lnk: &Path, tmp: &Path)
    -> Result<(), LinkError>
{
    if let Err(e) = hard_link(&tgt, &tmp) {
        return Err(LinkError::Create(e));
    }
    if let Err(e) = rename(&tmp, &lnk) {
        remove_file(&tmp).map_err(|e| LinkError::Remove(e))?;
        return Err(LinkError::Rename(e));
    }
    Ok(())
}
