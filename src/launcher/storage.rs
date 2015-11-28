use std::path::Path;
use std::io::{stdout, stderr, Write};

use argparse::{ArgumentParser, Store};

use file_util::{safe_ensure_dir, ensure_symlink};
use config::read_settings::MergedSettings;


pub fn init_dir(settings: &MergedSettings, project_root: &Path,
    args: Vec<String>)
    -> Result<i32, String>
{
    let mut name = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Inits the storage subdirectory to use for the containers. This is
            useful in build bots (for example gitlab-ci default agent) which
            try their best to entirely clean the project directory. So we need
            to create `.vagga` dir again and link it to the correct folder in
            the storage.
            ");
        ap.refer(&mut name)
            .add_argument("subdir_name", Store, "
                The name of a subdirectory to use for the project in the
                storage dir")
            .required();
        let mut args = args;
        args.insert(0, "vagga _init_storage_dir".to_string());
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => return Ok(x),
        }
    }
    if name.find('/').is_some() {
        writeln!(&mut stderr(), "Directory name must not contain slash").ok();
        return Ok(1);
    }
    if settings.storage_dir.is_none() {
        writeln!(&mut stderr(), "This command may only be run if storage-dir \
            is configured in settings").ok();
        return Ok(1);
    }
    let target = settings.storage_dir.as_ref().unwrap().join(name);
    try!(safe_ensure_dir(&target));
    let vagga = project_root.join(".vagga");
    try!(safe_ensure_dir(&vagga));
    let lnk = vagga.join(".lnk");
    try!(ensure_symlink(&target, &lnk)
        .map_err(|e| format!("Error symlinking {:?}: {}", lnk, e)));
    let target_lnk = target.join(".lnk");
    try!(ensure_symlink(&project_root, &target_lnk)
        .map_err(|e| format!("Error symlinking {:?}: {}", target_lnk, e)));

    return Ok(0);
}
