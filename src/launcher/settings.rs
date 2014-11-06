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
    pub allowed_dirs: TreeMap<String, Path>,
    pub allowed_files: TreeMap<String, Path>,
    pub storage_dir: Option<Path>,
    pub cache_dir: Option<Path>,
}

pub fn read_settings(project_root: &Path)
    -> Result<(MergedSettings, Settings), String>
{
    let mut ext_settings = MergedSettings {
        allowed_dirs: TreeMap::new(),
        allowed_files: TreeMap::new(),
        storage_dir: None,
        cache_dir: None,
    };
    let mut int_settings = Settings {
        version_check: true,
    };
    return Ok((ext_settings, int_settings));
}
