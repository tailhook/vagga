use std::io::{stdout, stderr};
use std::path::PathBuf;

use argparse::{ArgumentParser, Store, ParseOption};
use super::build_mode::{build_mode, BuildMode};
use super::compression_type::{compression_type, CompressionType};


pub struct Options {
    pub name: String,
    pub file: Option<PathBuf>,
    pub compression_type: Option<CompressionType>,
    pub build_mode: BuildMode,
}

impl Options {
    pub fn parse(args: &Vec<String>) -> Result<Options, i32> {
        let mut opt = Options {
            name: "".to_string(),
            file: None,
            compression_type: None,
            build_mode: BuildMode::NoImage,
        };
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("
                Packs image into tar archive.

                Unfortunately compression is not supported yet. It's
                recommended to stream archive to compressor.
                ");
            ap.refer(&mut opt.name)
                .add_argument("container_name", Store,
                    "Container name to pack");
            ap.refer(&mut opt.file)
                .add_option(&["-f", "--file"], ParseOption,
                    "File to store tar archive at");
            compression_type(&mut ap, &mut opt.compression_type);
            build_mode(&mut ap, &mut opt.build_mode);
            match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
                Ok(()) => {}
                Err(0) => return Err(0),
                Err(_) => {
                    return Err(122);
                }
            }
        }
        Ok(opt)
    }
}
