use std::os::homedir;
use std::io::fs::PathExtensions;
use std::default::Default;
use std::collections::TreeMap;

use config::Settings;
use quire::parse_config;
use quire::validate as V;


#[deriving(PartialEq, Decodable)]
struct SecureSettings {
    allowed_dirs: TreeMap<String, Path>,
    storage_dir: Option<Path>,
    cache_dir: Option<Path>,
    version_check: Option<bool>,
    site_settings: TreeMap<Path, SecureSettings>,
}

pub fn secure_settings_validator<'a>(has_children: bool)
    -> Box<V::Validator + 'a>
{
    let mut members = vec!(
        ("allowed_dirs".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("storage_dir".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("cache_dir".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("version_check".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
    );
    if has_children {
        members.push(
            ("site_settings".to_string(), box V::Mapping {
                key_element: box V::Scalar {
                    .. Default::default()} as Box<V::Validator>,
                value_element: secure_settings_validator(false),
                .. Default::default()} as Box<V::Validator>),
        );
    }
    return box V::Structure {
        members: members, .. Default::default()} as Box<V::Validator>;
}

#[deriving(PartialEq, Decodable)]
struct InsecureSettings {
    version_check: Option<bool>,
    shared_cache: Option<bool>,
}

pub fn insecure_settings_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Structure { members: vec!(
        ("version_check".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator + 'a>),
        ("shared_cache".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator + 'a>),
    ), .. Default::default()} as Box<V::Validator>;
}

struct MergedSettings {
    pub allowed_dirs: TreeMap<String, Path>,
    pub allowed_files: TreeMap<String, Path>,
    pub storage_dir: Option<Path>,
    pub cache_dir: Option<Path>,
    pub shared_cache: bool,
}

pub fn read_settings(project_root: &Path)
    -> Result<(MergedSettings, Settings), String>
{
    let mut ext_settings = MergedSettings {
        allowed_dirs: TreeMap::new(),
        allowed_files: TreeMap::new(),
        storage_dir: None,
        cache_dir: None,
        shared_cache: false,
    };
    let mut int_settings = Settings {
        version_check: true,
    };
    let mut secure_files = vec!();
    match homedir() {
        Some(home) => {
            secure_files.push(home.join(".config/vagga/settings.yaml"));
            secure_files.push(home.join(".vagga/settings.yaml"));
            secure_files.push(home.join(".vagga.yaml"));
        }
        None => {}
    }
    for filename in secure_files.iter() {
        let cfg: SecureSettings = try!(parse_config(filename,
            &*secure_settings_validator(true), Default::default()));
        for (k, v) in cfg.allowed_dirs.iter() {
            ext_settings.allowed_dirs.insert(k.clone(), v.clone());
        }
        if let Some(dir) = cfg.storage_dir {
            ext_settings.storage_dir = Some(dir);
        }
        if let Some(dir) = cfg.cache_dir {
            ext_settings.cache_dir = Some(dir);
            ext_settings.shared_cache = true;
        }
        if let Some(val) = cfg.version_check {
            int_settings.version_check = val;
        }
        if let Some(cfg) = cfg.site_settings.find(project_root) {
            for (k, v) in cfg.allowed_dirs.iter() {
                ext_settings.allowed_dirs.insert(k.clone(), v.clone());
            }
            if let Some(ref dir) = cfg.storage_dir {
                ext_settings.storage_dir = Some(dir.clone());
            }
            if let Some(ref dir) = cfg.cache_dir {
                ext_settings.cache_dir = Some(dir.clone());
                ext_settings.shared_cache = true;
            }
            if let Some(val) = cfg.version_check {
                int_settings.version_check = val;
            }
        }
    }
    let mut insecure_files = vec!();
    insecure_files.push(project_root.join(".vagga.settings.yaml"));
    insecure_files.push(project_root.join(".vagga/settings.yaml"));
    for filename in insecure_files.iter() {
        if !filename.exists() {
            continue;
        }
        let cfg: InsecureSettings = try!(parse_config(filename,
            &*insecure_settings_validator(), Default::default()));
        if let Some(val) = cfg.version_check {
            int_settings.version_check = val;
        }
        if let Some(val) = cfg.shared_cache {
            ext_settings.shared_cache = val;
        }
    }
    return Ok((ext_settings, int_settings));
}
