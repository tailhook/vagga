use argparse::{ArgumentParser, StoreConst};

#[cfg(feature="containers")]
use capsule::packages as capsule;


#[derive(Clone, Copy, Debug)]
pub enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
}

impl CompressionType {
    pub fn get_short_option(&self) -> &str {
        match *self {
            CompressionType::Gzip => "-z",
            CompressionType::Bzip2 => "-j",
            CompressionType::Xz => "-J",
        }
    }

    #[cfg(feature="containers")]
    pub fn get_capsule_feature(&self) -> capsule::Feature {
        match *self {
            CompressionType::Gzip => capsule::Gzip,
            CompressionType::Bzip2 => capsule::Bzip2,
            CompressionType::Xz => capsule::Xz,
        }
    }
}

pub fn compression_type<'x>(ap: &mut ArgumentParser<'x>,
    compression_type: &'x mut Option<CompressionType>)
{
    ap.refer(compression_type)
    .add_option(&["-z", "--gzip"], StoreConst(Some(CompressionType::Gzip)),
        "Filter the image through gzip.")
    .add_option(&["-j", "--bzip2"], StoreConst(Some(CompressionType::Bzip2)),
        "Filter the image through bzip2.")
    .add_option(&["-J", "--xz"], StoreConst(Some(CompressionType::Xz)),
        "Filter the image through xz.");
}
