#[macro_use]
extern crate error_chain;
extern crate xmltree;
extern crate rayon;
extern crate chrono;
extern crate geo;

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use rayon::prelude::*;
use chrono::prelude::*;
use geo::length::Length;

mod errors;
use errors::*;

pub struct TrackPoint {
    point: geo::Point<f64>,
    elevation_meters: f64,
    time: DateTime<Utc>,
}
impl TrackPoint {
    fn from_xml_elem(elem: &xmltree::Element) -> TrackPoint {
        TrackPoint {
            point: geo::Point::new(
                elem.attributes.get("lat").unwrap().parse().unwrap(),
                elem.attributes.get("lon").unwrap().parse().unwrap(),
            ),
            elevation_meters: elem.get_child("ele")
                .unwrap()
                .text
                .clone()
                .unwrap()
                .parse()
                .unwrap(),
            time: elem.get_child("time")
                .unwrap()
                .text
                .clone()
                .unwrap()
                .parse()
                .unwrap(),
        }
    }
}

pub struct Gpx {
    time: DateTime<Utc>,
    pub track_points: Vec<TrackPoint>,
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
                .unwrap()
                .parse()
                .unwrap(),
            track_points: element_get_path(&document, &["trk", "trkseg"])
                .unwrap()
                .children
                .par_iter()
                .filter(|elem| elem.name == "trkpt")
                .map(|elem| TrackPoint::from_xml_elem(elem))
                .collect(),
        })
    }
    fn as_line_string(&self) -> geo::LineString<f64> {
        geo::LineString(self.track_points.iter().map(|x| x.point).collect())
    }
    pub fn distance_meters(&self) -> f64 {
        self.as_line_string().length() * 100.0 * 1000.0
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
