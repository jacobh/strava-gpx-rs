#[macro_use]
extern crate error_chain;
extern crate xmltree;

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};

mod errors;
use errors::*;

pub struct TrackPoint {
    lat: f64,
    lon: f64,
    elevation_meters: f64,
}
impl TrackPoint {
    fn from_xml_elem(elem: &xmltree::Element) -> TrackPoint {
        TrackPoint {
            lat: elem.attributes.get("lat").unwrap().parse().unwrap(),
            lon: elem.attributes.get("lon").unwrap().parse().unwrap(),
            elevation_meters: elem.get_child("ele").unwrap().text.clone().unwrap().parse().unwrap(),
        }
    }
}

pub struct Gpx {
    time: String,
    pub track_points: Vec<TrackPoint>,
    document: xmltree::Element,
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
        let document = xmltree::Element::parse(r)?;
        Ok(Gpx {
            time: element_get_path(&document, &["metadata", "time"])
                .unwrap()
                .text
                .clone()
                .unwrap(),
            track_points: element_get_path(&document, &["trk", "trkseg"])
                .unwrap()
                .children
                .iter()
                .filter(|elem| elem.name == "trkpt")
                .map(|elem| TrackPoint::from_xml_elem(elem))
                .collect(),
            document: document,
        })
    }
}

fn element_get_path<'a>(elem: &'a xmltree::Element, path: &[&str]) -> Option<&'a xmltree::Element> {
    match path.split_first() {
        Some((child_name, path)) => {
            match elem.get_child(*child_name) {
                Some(child) => element_get_path(child, path),
                None => None,
            }
        }
        None => Some(elem),
    }
}
