use std::env;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use quire::{parse_config, parse_string, Options};
use quire::validate as V;

use crate::builder::commands::docker::DEFAULT_REGISTRY_HOST;
use crate::config::Settings;
use crate::path_util::Expand;


#[derive(PartialEq, Deserialize, Debug)]
struct SecureSettings {
    storage_dir: Option<PathBuf>,
    storage_subdir_from_env_var: Option<String>,
    cache_dir: Option<PathBuf>,
    version_check: Option<bool>,
    proxy_env_vars: Option<bool>,
    ubuntu_mirror: Option<String>,
    ubuntu_skip_locking: Option<bool>,
    versioned_build_dir: Option<bool>,
    alpine_mirror: Option<String>,
    #[serde(default)]
    site_settings: BTreeMap<PathBuf, SecureSettings>,
    external_volumes: HashMap<String, PathBuf>,
    push_image_script: Option<String>,
    build_lock_wait: Option<bool>,
    auto_apply_sysctl: Option<bool>,
    index_all_images: Option<bool>,
    hard_link_identical_files: Option<bool>,
    hard_link_between_projects: Option<bool>,
    run_symlinks_as_commands: Option<bool>,
    disable_auto_clean: Option<bool>,
    environ: BTreeMap<String, String>,
    propagate_environ: BTreeSet<String>,
    docker_insecure_registries: HashSet<String>,
    docker_registry_aliases: HashMap<String, String>,
}

pub fn secure_settings_validator<'a>(has_children: bool)
    -> V::Structure<'a>
{
    let mut s = V::Structure::new()
        .member("storage_dir", V::Scalar::new().optional())
        .member("storage_subdir_from_env_var", V::Scalar::new().optional())
        .member("cache_dir", V::Scalar::new().optional())
        .member("version_check", V::Scalar::new().optional())
        .member("proxy_env_vars", V::Scalar::new().optional())
        .member("ubuntu_mirror", V::Scalar::new().optional())
        .member("ubuntu_skip_locking", V::Scalar::new().optional())
        .member("versioned_build_dir", V::Scalar::new().optional())
        .member("alpine_mirror", V::Scalar::new().optional())
        .member("external_volumes", V::Mapping::new(
            V::Directory::new().absolute(false),
            V::Directory::new().absolute(true)))
        .member("push_image_script", V::Scalar::new().optional())
        .member("build_lock_wait", V::Scalar::new().optional())
        .member("auto_apply_sysctl", V::Scalar::new().optional())
        .member("index_all_images", V::Scalar::new().optional())
        .member("hard_link_identical_files", V::Scalar::new().optional())
        .member("hard_link_between_projects",
            V::Scalar::new().optional())
        .member("run_symlinks_as_commands", V::Scalar::new().optional())
        .member("disable_auto_clean", V::Scalar::new().optional())
        .member("environ", V::Mapping::new(
            V::Scalar::new(), V::Scalar::new()))
        .member("propagate_environ", V::Sequence::new(V::Scalar::new()))
        .member("docker_insecure_registries", V::Sequence::new(V::Scalar::new()))
        .member("docker_registry_aliases", V::Mapping::new(
            V::Scalar::new(), V::Scalar::new()));
    if has_children {
        s = s.member("site_settings", V::Mapping::new(
            V::Scalar::new(),
            secure_settings_validator(false)));
    }
    return s;
}

#[derive(PartialEq, Deserialize)]
struct InsecureSettings {
    version_check: Option<bool>,
    shared_cache: Option<bool>,
    ubuntu_mirror: Option<String>,
    ubuntu_skip_locking: Option<bool>,
    versioned_build_dir: Option<bool>,
    alpine_mirror: Option<String>,
    build_lock_wait: Option<bool>,
    run_symlinks_as_commands: Option<bool>,
    disable_auto_clean: Option<bool>,
    environ: BTreeMap<String, String>,
}

pub fn insecure_settings_validator<'a>() -> Box<dyn V::Validator + 'a> {
    Box::new(V::Structure::new()
    .member("version_check", V::Scalar::new().optional())
    .member("shared_cache", V::Scalar::new().optional())
    .member("ubuntu_mirror", V::Scalar::new().optional())
    .member("ubuntu_skip_locking", V::Scalar::new().optional())
    .member("versioned_build_dir", V::Scalar::new().optional())
    .member("alpine_mirror", V::Scalar::new().optional())
    .member("run_symlinks_as_commands", V::Scalar::new().optional())
    .member("disable_auto_clean", V::Scalar::new().optional())
    .member("environ", V::Mapping::new(
        V::Scalar::new(), V::Scalar::new())))
}

#[derive(Debug)]
pub struct MergedSettings {
    pub external_volumes: HashMap<String, PathBuf>,
    pub push_image_script: Option<String>,
    pub storage_dir: Option<PathBuf>,
    pub storage_subdir_from_env_var: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub shared_cache: bool,
    pub disable_auto_clean: bool,
    pub propagate_environ: BTreeSet<String>,
}

fn merge_settings(cfg: SecureSettings, project_root: &Path,
    ext_settings: &mut MergedSettings, int_settings: &mut Settings)
    -> Result<(), String>
{
    if let Some(dir) = cfg.storage_dir {
        ext_settings.storage_dir = Some(dir.expand_home()
            .map_err(|()| format!("Can't expand tilde `~` in storage dir \
                no HOME found"))?);
    }
    if let Some(name) = cfg.storage_subdir_from_env_var {
        ext_settings.storage_subdir_from_env_var = Some(name.clone());
        int_settings.storage_subdir_from_env_var = Some(name.clone());
    }
    if let Some(dir) = cfg.cache_dir {
        ext_settings.cache_dir = Some(dir.expand_home()
            .map_err(|()| format!("Can't expand tilde `~` in cache dir \
                no HOME found"))?);
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
    if let Some(val) = cfg.ubuntu_skip_locking {
        int_settings.ubuntu_skip_locking = val;
    }
    if let Some(val) = cfg.versioned_build_dir {
        int_settings.versioned_build_dir = val;
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
    if let Some(val) = cfg.index_all_images {
        int_settings.index_all_images = val;
    }
    if let Some(val) = cfg.hard_link_identical_files {
        int_settings.hard_link_identical_files = val;
    }
    if let Some(val) = cfg.hard_link_between_projects {
        int_settings.hard_link_between_projects = val;
    }
    if let Some(val) = cfg.run_symlinks_as_commands {
        int_settings.run_symlinks_as_commands = val;
    }
    if let Some(val) = cfg.disable_auto_clean {
        int_settings.disable_auto_clean = val;
    }
    for (k, v) in &cfg.environ {
        int_settings.environ.insert(k.clone(), v.clone());
    }
    for item in &cfg.propagate_environ {
        ext_settings.propagate_environ.insert(item.clone());
    }
    for registry in &cfg.docker_insecure_registries {
        int_settings.docker_insecure_registries.insert(registry.clone());
    }
    for (alias, registry) in &cfg.docker_registry_aliases {
        int_settings.docker_registry_aliases.insert(alias.clone(), registry.clone());
    }
    if let Some(cfg) = cfg.site_settings.get(project_root) {
        if let Some(ref dir) = cfg.storage_dir {
            ext_settings.storage_dir = Some(dir.clone());
        }
        if let Some(ref name) = cfg.storage_subdir_from_env_var {
            ext_settings.storage_subdir_from_env_var = Some(name.clone());
            int_settings.storage_subdir_from_env_var = Some(name.clone());
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
        if let Some(val) = cfg.ubuntu_skip_locking {
            int_settings.ubuntu_skip_locking = val;
        }
        if let Some(val) = cfg.versioned_build_dir {
            int_settings.versioned_build_dir = val;
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
        if let Some(val) = cfg.index_all_images {
            int_settings.index_all_images = val;
        }
        if let Some(val) = cfg.hard_link_identical_files {
            int_settings.hard_link_identical_files = val;
        }
        if let Some(val) = cfg.hard_link_between_projects {
            int_settings.hard_link_between_projects = val;
        }
        if let Some(val) = cfg.run_symlinks_as_commands {
            int_settings.run_symlinks_as_commands = val;
        }
        if let Some(val) = cfg.disable_auto_clean {
            int_settings.disable_auto_clean = val;
        }
        for (k, v) in &cfg.environ {
            int_settings.environ.insert(k.clone(), v.clone());
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
        storage_subdir_from_env_var: None,
        cache_dir: None,
        shared_cache: false,
        disable_auto_clean: false,
        propagate_environ: BTreeSet::new(),
    };
    let mut int_settings = Settings {
        proxy_env_vars: true,
        version_check: true,
        uid_map: None,
        ubuntu_mirror: None,
        ubuntu_skip_locking: false,
        versioned_build_dir: false,
        alpine_mirror: None,
        push_image_script: None,
        build_lock_wait: false,
        auto_apply_sysctl: false,
        index_all_images: false,
        hard_link_identical_files: false,
        hard_link_between_projects: false,
        run_symlinks_as_commands: true,
        disable_auto_clean: false,
        environ: BTreeMap::new(),
        storage_subdir_from_env_var: None,
        docker_insecure_registries: {
            let mut registries = HashSet::new();
            registries.insert("localhost".to_string());
            registries
        },
        docker_registry_aliases: {
            let mut aliases = HashMap::new();
            aliases.insert("docker.io".to_string(), DEFAULT_REGISTRY_HOST.to_string());
            aliases
        },
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
        let cfg: SecureSettings = parse_config(filename,
            &secure_settings_validator(true), &Options::default())
            .map_err(|e| format!("{}", e))?;
        merge_settings(cfg, &project_root,
            &mut ext_settings, &mut int_settings)?
    }
    if let Ok(settings) = env::var("VAGGA_SETTINGS") {
        let cfg: SecureSettings = parse_string("<env:VAGGA_SETTINGS>",
                &settings,
                &secure_settings_validator(true), &Options::default())
            .map_err(|e| format!("{}", e))?;
        merge_settings(cfg, &project_root,
            &mut ext_settings, &mut int_settings)?
    }
    let mut insecure_files = vec!();
    insecure_files.push(project_root.join(".vagga.settings.yaml"));
    insecure_files.push(project_root.join(".vagga/settings.yaml"));
    for filename in insecure_files.iter() {
        if !filename.exists() {
            continue;
        }
        let cfg: InsecureSettings = parse_config(filename,
            &*insecure_settings_validator(), &Options::default())
            .map_err(|e| format!("{}", e))?;
        if let Some(val) = cfg.version_check {
            int_settings.version_check = val;
        }
        if let Some(ref val) = cfg.ubuntu_mirror {
            int_settings.ubuntu_mirror = Some(val.clone());
        }
        if let Some(val) = cfg.ubuntu_skip_locking {
            int_settings.ubuntu_skip_locking = val;
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
        if let Some(val) = cfg.run_symlinks_as_commands {
            int_settings.run_symlinks_as_commands = val;
        }
        for (k, v) in &cfg.environ {
            int_settings.environ.insert(k.clone(), v.clone());
        }
    }
    return Ok((ext_settings, int_settings));
}
