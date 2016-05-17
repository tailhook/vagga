use std::path::Path;

use builder::context::Context;
use builder::distrib::{Unknown,Distribution};
use builder::error::{Error};
use builder::commands::{composer, gem, npm, pip};
use builder::packages;
use build_step::BuildStep;
use container::util::clean_dir;
use container::mount::{unmount, mount_system_dirs, mount_proc};
use file_util::{create_dir, copy};


pub struct Guard<'a> {
    pub ctx: Context<'a>,
    pub distro: Box<Distribution>,
}


impl<'a> Guard<'a> {
    pub fn build(ctx: Context) -> Result<(), Error> {
        Guard {
            ctx: ctx,
            distro: Box::new(Unknown),
        }.run_process()
    }

    fn run_process(&mut self) -> Result<(), Error> {
        try!(self.start());

        for b in self.ctx.container_config.setup.iter() {
            debug!("Building step: {:?}", b);
            try!(self.ctx.timelog.mark(format_args!("Step: {:?}", b))
                .map_err(|e| format!("Can't write timelog: {}", e)));
            try!(b.build(self, true)
                .map_err(|e| Error::Step(b.0.clone(), e)));
        }

        try!(self.finish());
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        try!(mount_system_dirs());
        try!(mount_proc(&Path::new("/proc")));
        try_msg!(create_dir("/vagga/root/etc", false),
             "Error creating /etc dir: {err}");
        try!(copy("/etc/resolv.conf", "/vagga/root/etc/resolv.conf")
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));
        try!(self.ctx.timelog.mark(format_args!("Prepare"))
            .map_err(|e| format!("Can't write timelog: {}", e)));
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), String> {
        // Pip
        if self.ctx.featured_packages.contains(&packages::PipPy2) ||
           self.ctx.featured_packages.contains(&packages::PipPy3)
        {
            try!(pip::freeze(&mut self.ctx));
        }
        // Npm
        if self.ctx.featured_packages.contains(&packages::Npm) {
            try!(npm::list(&mut self.ctx));
        }
        // Composer
        if self.ctx.featured_packages.contains(&packages::Composer) {
            try!(composer::finish(&mut self.ctx));
        }
        // Gem
        if self.ctx.featured_packages.contains(&packages::Bundler) {
            try!(gem::list(&mut self.ctx));
        }

        try!(self.distro.finish(&mut self.ctx));

        let base = Path::new("/vagga/root");

        for path in self.ctx.mounted.iter().rev() {
            try!(unmount(path));
        }

        for dir in self.ctx.remove_dirs.iter() {
            try!(clean_dir(&base.join(dir), true)
                .map_err(|e| format!("Error removing dir: {}", e)));
        }

        for dir in self.ctx.empty_dirs.iter() {
            try!(clean_dir(&base.join(dir), false));
        }

        for dir in self.ctx.ensure_dirs.iter() {
            let fulldir = base.join(dir);
            try_msg!(create_dir(&fulldir, true),
                "Error creating dir: {err}");
        }

        try!(self.ctx.timelog.mark(format_args!("Finish"))
            .map_err(|e| format!("Can't write timelog: {}", e)));

        return Ok(());
    }
}
