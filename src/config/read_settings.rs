use std::env;
use std::default::Default;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use quire::{parse_config, parse_string};
use quire::validate as V;

use config::Settings;
use path_util::Expand;


#[derive(PartialEq, RustcDecodable, Debug)]
struct SecureSettings {
    storage_dir: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    version_check: Option<bool>,
    proxy_env_vars: Option<bool>,
    ubuntu_mirror: Option<String>,
    alpine_mirror: Option<String>,
    site_settings: BTreeMap<PathBuf, SecureSettings>,
    external_volumes: HashMap<String, PathBuf>,
    push_image_script: Option<String>,
    build_lock_wait: Option<bool>,
    auto_apply_sysctl: Option<bool>,
}

pub fn secure_settings_validator<'a>(has_children: bool)
    -> V::Structure<'a>
{
    let mut s = V::Structure::new()
        .member("storage_dir", V::Scalar::new().optional())
        .member("cache_dir", V::Scalar::new().optional())
        .member("version_check", V::Scalar::new().optional())
        .member("proxy_env_vars", V::Scalar::new().optional())
        .member("ubuntu_mirror", V::Scalar::new().optional())
        .member("alpine_mirror", V::Scalar::new().optional())
        .member("external_volumes", V::Mapping::new(
            V::Directory::new().is_absolute(false),
            V::Directory::new().is_absolute(true)))
        .member("push_image_script", V::Scalar::new().optional())
        .member("build_lock_wait", V::Scalar::new().optional())
        .member("auto_apply_sysctl", V::Scalar::new().optional());
    if has_children {
        s = s.member("site_settings", V::Mapping::new(
            V::Scalar::new(),
            secure_settings_validator(false)));
    }
    return s;
}

#[derive(PartialEq, RustcDecodable)]
struct InsecureSettings {
    version_check: Option<bool>,
    shared_cache: Option<bool>,
    ubuntu_mirror: Option<String>,
    alpine_mirror: Option<String>,
    build_lock_wait: Option<bool>,
}

pub fn insecure_settings_validator<'a>() -> Box<V::Validator + 'a> {
    Box::new(V::Structure::new()
    .member("version_check", V::Scalar::new().optional())
    .member("shared_cache", V::Scalar::new().optional())
    .member("ubuntu_mirror", V::Scalar::new().optional())
    .member("alpine_mirror", V::Scalar::new().optional()))
}

#[derive(Debug)]
pub struct MergedSettings {
    pub external_volumes: HashMap<String, PathBuf>,
    pub push_image_script: Option<String>,
    pub storage_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub shared_cache: bool,
}

fn merge_settings(cfg: SecureSettings, project_root: &Path,
    ext_settings: &mut MergedSettings, int_settings: &mut Settings)
    -> Result<(), String>
{
    if let Some(dir) = cfg.storage_dir {
        ext_settings.storage_dir = Some(try!(dir.expand_home()
            .map_err(|()| format!("Can't expand tilde `~` in storage dir \
                no HOME found"))));
    }
    if let Some(dir) = cfg.cache_dir {
        ext_settings.cache_dir = Some(try!(dir.expand_home()
            .map_err(|()| format!("Can't expand tilde `~` in cache dir \
                no HOME found"))));
        ext_settings.shared_cache = true;
    }
    if let Some(val) = cfg.version_check {
        int_settings.version_check = val;
    }
    if let Some(val) = cfg.proxy_env_vars {
        int_settings.proxy_env_vars = val;
    }
    if let Some(ref val) = cfg.ubuntu_mirror {
        int_settings.ubuntu_mirror = Some(val.clone());
    }
    if let Some(ref val) = cfg.alpine_mirror {
        int_settings.alpine_mirror = Some(val.clone());
    }
    for (k, v) in &cfg.external_volumes {
        ext_settings.external_volumes.insert(k.clone(), v.clone());
    }
    if let Some(ref val) = cfg.push_image_script {
        int_settings.push_image_script = Some(val.clone());
    }
    if let Some(val) = cfg.build_lock_wait {
        int_settings.build_lock_wait = val;
    }
    if let Some(val) = cfg.auto_apply_sysctl {
        int_settings.auto_apply_sysctl = val;
    }
    if let Some(cfg) = cfg.site_settings.get(project_root) {
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
        if let Some(ref val) = cfg.ubuntu_mirror {
            int_settings.ubuntu_mirror = Some(val.clone());
        }
        if let Some(ref val) = cfg.alpine_mirror {
            int_settings.alpine_mirror = Some(val.clone());
        }
        for (k, v) in &cfg.external_volumes {
            ext_settings.external_volumes.insert(k.clone(), v.clone());
        }
        if let Some(ref val) = cfg.push_image_script {
            int_settings.push_image_script = Some(val.clone());
        }
        if let Some(val) = cfg.build_lock_wait {
            int_settings.build_lock_wait = val;
        }
        if let Some(val) = cfg.auto_apply_sysctl {
            int_settings.auto_apply_sysctl = val;
        }
    }
    Ok(())
}

pub fn read_settings(project_root: &Path)
    -> Result<(MergedSettings, Settings), String>
{
    let mut ext_settings = MergedSettings {
        external_volumes: HashMap::new(),
        push_image_script: None,
        storage_dir: None,
        cache_dir: None,
        shared_cache: false,
    };
    let mut int_settings = Settings {
        proxy_env_vars: true,
        version_check: true,
        uid_map: None,
        ubuntu_mirror: None,
        alpine_mirror: None,
        push_image_script: None,
        build_lock_wait: false,
        auto_apply_sysctl: false,
    };
    let mut secure_files = vec!();
    if let Ok(home) = env::var("_VAGGA_HOME") {
        let home = Path::new(&home);
        secure_files.push(home.join(".config/vagga/settings.yaml"));
        secure_files.push(home.join(".vagga/settings.yaml"));
        secure_files.push(home.join(".vagga.yaml"));
    };
    for filename in secure_files.iter() {
        if !filename.exists() {
            continue;
        }
        let cfg: SecureSettings = try!(parse_config(filename,
            &secure_settings_validator(true), Default::default()));
        try!(merge_settings(cfg, &project_root,
            &mut ext_settings, &mut int_settings))
    }
    if let Ok(settings) = env::var("VAGGA_SETTINGS") {
        let cfg: SecureSettings = try!(parse_string("<env:VAGGA_SETTINGS>",
                &settings,
                &secure_settings_validator(true), Default::default())
            .map_err(|lst| lst.iter()
                           .map(ToString::to_string).collect::<Vec<_>>()[..]
                           .join("\n")));
        try!(merge_settings(cfg, &project_root,
            &mut ext_settings, &mut int_settings))
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
        if let Some(ref val) = cfg.ubuntu_mirror {
            int_settings.ubuntu_mirror = Some(val.clone());
        }
        if let Some(ref val) = cfg.alpine_mirror {
            int_settings.alpine_mirror = Some(val.clone());
        }
        if let Some(val) = cfg.shared_cache {
            ext_settings.shared_cache = val;
        }
        if let Some(val) = cfg.build_lock_wait {
            int_settings.build_lock_wait = val;
        }
    }
    return Ok((ext_settings, int_settings));
}
