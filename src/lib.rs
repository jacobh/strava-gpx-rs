#[macro_use]
extern crate error_chain;
extern crate xmltree;

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};

mod errors;
use errors::*;

pub struct Gpx {
    document: xmltree::Element
}
impl Gpx {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Gpx> {
        let reader = {
            let f = File::open(path)?;
            BufReader::new(f)
        };
        Self::parse(reader)
    }
    pub fn parse<R: Read>(r: R) -> Result<Gpx> {
        Ok(Gpx {
            document: xmltree::Element::parse(r)?
        })
    }
}
