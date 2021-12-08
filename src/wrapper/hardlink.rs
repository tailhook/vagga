use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};

use crate::container::util::{
    check_signature,
    collect_containers_from_storage,
    collect_container_dirs,
    hardlink_all_identical_files,
    version_from_symlink,
    write_container_signature,
};
use crate::file_util::human_size;

use super::{Wrapper, setup};


pub fn verify_cmd(wrapper: &Wrapper, args: Vec<String>)
    -> Result<i32, String>
{
    let mut container = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Verifies container files checksum");
        ap.refer(&mut container)
            .add_argument("container", Store, "Container to verify");
        ap.stop_on_first_argument(true);
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {},
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }

    let vagga_dir = wrapper.project_root.join(".vagga");
    let ver = version_from_symlink(vagga_dir.join(&container))?;

    let cont_dir = setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings)?
    .map(|d| d.join(".roots").join(&ver))
    .ok_or_else(|| format!("Cannot detect base vagga directory"))?;

    println!("Checking container: {:?}", &cont_dir);
    match check_signature(&cont_dir) {
        Ok(None) => {
            println!("Ok");
            Ok(0)
        },
        Ok(Some(ref diff)) => {
            println!("Container is corrupted");
            if !diff.missing_paths.is_empty() {
                println!("Missing paths:");
                for p in &diff.missing_paths {
                    println!("\t{}", p.to_string_lossy());
                }
            }
            if !diff.extra_paths.is_empty() {
                println!("Extra paths:");
                for p in &diff.extra_paths {
                    println!("\t{}", p.to_string_lossy());
                }
            }
            if !diff.corrupted_paths.is_empty() {
                println!("Corrupted paths:");
                for p in &diff.corrupted_paths {
                    println!("\t{}", p.to_string_lossy());
                }
            }
            Ok(1)
        },
        Err(e) => Err(format!("Error checking container signature: {}", e)),
    }
}

pub fn hardlink_cmd(wrapper: &Wrapper, args: Vec<String>)
    -> Result<i32, String>
{
    let mut global = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Indexes and hardlinks containers");
        ap.refer(&mut global)
            .add_option(&["--global"], StoreTrue,
                        "Hardlink containers between projects.");
        ap.stop_on_first_argument(true);
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {},
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }

    let _cont_dirs = if global {
        if let Some(ref storage_dir) = wrapper.ext_settings.storage_dir {
            collect_containers_from_storage(storage_dir)?
        } else {
            return Err(format!(
                "The --global flag is only meaningful if you configure \
                 storage-dir in settings"));
        }
    } else {
        let roots = setup::get_vagga_base(
            wrapper.project_root, wrapper.ext_settings)?
        .map(|x| x.join(".roots"))
        .ok_or_else(|| format!("Cannot detect base vagga directory"))?;
        collect_container_dirs(&roots, None)?
    };
    let mut cont_dirs = _cont_dirs.iter().collect::<Vec<_>>();
    cont_dirs.sort_by_key(|d| (&d.project, &d.name, d.modified));

    for cont_dir in &cont_dirs {
        let index_path = cont_dir.path.join("index.ds1");
        if !index_path.exists() {
            warn!("Indexing container {:?} ...", &cont_dir.path);
            write_container_signature(&cont_dir.path)?;
        }
    }

    match hardlink_all_identical_files(cont_dirs.iter().map(|d| &d.path)) {
        Ok((count, size)) => {
            warn!("Found and linked {} ({}) identical files",
                  count, human_size(size));
            Ok(0)
        },
        Err(msg) => {
            Err(format!("Error when linking container files: {}", msg))
        },
    }
}
