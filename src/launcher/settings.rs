use config::Settings;
use std::collections::TreeMap;


#[deriving(PartialEq, Decodable)]
struct SecureSettings {
    allowed_dirs: TreeMap<String, Path>,
    allowed_files: TreeMap<String, Path>,
    storage_dir: Option<Path>,
    cache_dir: Option<Path>,
    version_check: Option<bool>,
    site_settings: TreeMap<String, SecureSettings>,
}

#[deriving(PartialEq, Decodable)]
struct InsecureSettings {
    storage_dir: Option<Path>,
    cache_dir: Option<Path>,
    version_check: Option<bool>,
    site_settings: TreeMap<String, InsecureSettings>,
}

struct MergedSettings {
    allowed_dirs: TreeMap<String, Path>,
    allowed_files: TreeMap<String, Path>,
    storage_dir: Path,
    cache_dir: Path,
    version_check: bool,
}

pub fn read_settings(project_root: &Path)
    -> Result<(MergedSettings, Settings), String>
{
    unimplemented!();
}
