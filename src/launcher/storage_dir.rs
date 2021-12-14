use std::env;
use std::path::PathBuf;

use crate::launcher::Context;
use crate::storage_dir::sanitize;


pub fn get_base(ctx: &Context) -> Option<PathBuf> {
    if let Some(ref storage_base) = ctx.ext_settings.storage_dir {
        let path = ctx.config_dir.join(".vagga/.lnk");
        if !path.exists() {
            if let Some(ref v) = ctx.ext_settings.storage_subdir_from_env_var {
                if let Ok(value) = env::var(v) {
                    let sanitized = sanitize(&value);
                    Some(storage_base.join(sanitized))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            Some(path) // TODO(tailhook) or resolve the link?
        }
    } else {
        Some(ctx.config_dir.join(".vagga"))
    }
}
