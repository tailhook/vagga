use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, PushConst, StoreTrue};

use super::setup;
use super::Wrapper;

#[derive(Copy)]
enum Action {
    Temporary,
    Old,
    Everything,
    Orphans,
}


pub fn clean_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<isize, String>
{
    let mut global = false;
    let mut actions = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Performs various cleanup tasks
            ");
        ap.refer(&mut actions)
          .add_option(&["--tmp", "--tmp-folders"],
                Box::new(PushConst(Action::Temporary)),
                "Clean temporary containers (failed builds)")
          .add_option(&["--old", "--old-containers"],
                Box::new(PushConst(Action::Old)),
                "Clean old versions of containers (configurable)")
          .add_option(&["--everything"],
                Box::new(PushConst(Action::Everything)),
                "Clean whole `.vagga` folder. Useful when deleting a project.
                 With ``--global`` cleans whole storage-dir and cache-dir")
          .add_option(&["--orphans"],
                Box::new(PushConst(Action::Orphans)),
                "Without `--global` removes containers which are not in
                 vagga.yaml any more. With `--global` removes all folders
                 which have `.lnk` pointing to nowhere (i.e. project dir
                 already deleted while vagga folder is not)")
          .required();
        ap.refer(&mut global)
          .add_option(&["--global"], Box::new(StoreTrue),
                "Apply cleanup command to all containers. Works only \
                if `storage-dir` is configured in settings");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));
    unimplemented!();
}
