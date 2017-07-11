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
    fn as_points(&self) -> Vec<&geo::Point<f64>> {
        self.track_points.iter().map(|x| &x.point).collect()
    }
    pub fn distance_meters(&self) -> f64 {
        let mut iter = self.as_points().into_iter().peekable();
        let mut distance: f64 = 0.0;
        loop {
            match iter.next() {
                Some(point) => {
                    match iter.peek() {
                        Some(next_point) => {
                            distance += haversine(point, next_point);
                        }
                        None => return distance
                    }
                }
                None => return distance
            }
        }
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

static R: f64 = 6371.0;

fn haversine(p1: &geo::Point<f64>, p2: &geo::Point<f64>) -> f64 {
    let lat1 = p1.x().to_radians();
    let lon1 = p1.y().to_radians();
    let lat2 = p2.x().to_radians();
    let lon2 = p2.y().to_radians();

    let dlon = lon2 - lon1;
    let dlat = lat2 - lat1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon/2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    c * R
}
