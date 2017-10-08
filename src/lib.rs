extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate geo;
extern crate itertools;
extern crate rayon;
extern crate xmltree;

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use rayon::prelude::*;
use chrono::prelude::*;
use chrono::Duration;
use geo::length::Length;
use geo::distance::Distance;
use geo::contains::Contains;
use itertools::Itertools;

mod errors;
use errors::*;

type Celsius = f64;
type Meters = f64;
type BeatsPerMinute = f64;
type RevolutionsPerMinute = f64;

pub struct TrackPoint {
    pub point: geo::Point<f64>,
    elevation_meters: f64,
    pub time: DateTime<Utc>,
    extensions: Vec<TrackPointExtension>,
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
                .as_ref()
                .unwrap()
                .parse()
                .unwrap(),
            time: elem.get_child("time")
                .unwrap()
                .text
                .as_ref()
                .unwrap()
                .parse()
                .unwrap(),
            extensions: element_get_path(&elem, &["extensions", "gpxtpx:TrackPointExtension"])
                .map(|elem| {
                    elem.children
                        .iter()
                        .map(|child| TrackPointExtension::from_xml_elem(child))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

pub enum TrackPointExtension {
    AirTemperature(Celsius),
    WaterTemperature(Celsius),
    Depth(Meters),
    HeartRate(BeatsPerMinute),
    Cadence(RevolutionsPerMinute),
}
impl TrackPointExtension {
    fn from_xml_elem(elem: &xmltree::Element) -> TrackPointExtension {
        let elem_text = elem.text.as_ref().unwrap();
        match elem.name.as_str() {
            "gpxtpx:atemp" => TrackPointExtension::AirTemperature(elem_text.parse().unwrap()),
            "gpxtpx:wtemp" => TrackPointExtension::WaterTemperature(elem_text.parse().unwrap()),
            "gpxtpx:depth" => TrackPointExtension::Depth(elem_text.parse().unwrap()),
            "gpxtpx:hr" => TrackPointExtension::HeartRate(elem_text.parse().unwrap()),
            "gpxtpx:cad" => TrackPointExtension::Cadence(elem_text.parse().unwrap()),
            _ => unimplemented!(),
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
                .as_ref()
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
}

fn element_get_path<'a>(elem: &'a xmltree::Element, path: &[&str]) -> Option<&'a xmltree::Element> {
    match path.split_first() {
        Some((child_name, path)) => match elem.get_child(*child_name) {
            Some(child) => element_get_path(child, path),
            None => None,
        },
        None => Some(elem),
    }
}

pub trait TrackPointCollection {
    fn get_track_points(&self) -> &Vec<TrackPoint>;
    fn as_line_string(&self) -> geo::LineString<f64> {
        geo::LineString(self.get_track_points().iter().map(|x| x.point).collect())
    }
    fn distance_meters(&self) -> Meters {
        self.as_line_string().length() * 100.0 * 1000.0
    }
    fn duration(&self) -> Duration {
        let track_points = self.get_track_points();
        track_points[track_points.len() - 1]
            .time
            .signed_duration_since(track_points[0].time)
    }
    fn total_elevation_gain_meters(&self) -> Meters {
        self.get_track_points()
            .iter()
            .tuple_windows()
            .fold(0.0, |acc, (p1, p2)| {
                let gain = p2.elevation_meters - p1.elevation_meters;
                if gain > 0.0 {
                    acc + gain
                } else {
                    acc
                }
            })
    }
    fn as_speed_meters_per_sec(&self) -> Vec<f64> {
        self.get_track_points()
            .iter()
            .tuple_windows()
            .map(|(p1, p2)| {
                let distance_meters = p1.point.distance(&p2.point) * 100.0 * 1000.0;
                let secs = p2.time.signed_duration_since(p1.time).num_seconds() as f64;
                distance_meters / secs
            })
            .collect()
    }
    fn max_distance_apart(&self, other_tpc: &Self) -> Meters {
        let line_string = self.as_line_string();
        other_tpc
            .get_track_points()
            .par_iter()
            .map(|x| line_string.distance(&x.point))
            .reduce_with(|a, b| if a > b { a } else { b })
            .unwrap() * 100.0 * 1000.0
    }
}
impl TrackPointCollection for Vec<TrackPoint> {
    fn get_track_points(&self) -> &Vec<TrackPoint> {
        self
    }
}
impl TrackPointCollection for Gpx {
    fn get_track_points(&self) -> &Vec<TrackPoint> {
        &self.track_points
    }
}

pub struct Circle {
    centroid: geo::Point<f64>,
    radius: Meters,
}
impl Circle {
    pub fn new(lat: f64, lon: f64, radius: Meters) -> Circle {
        Circle {
            centroid: geo::Point::new(lat, lon),
            radius: radius,
        }
    }
}
impl Contains<geo::Point<f64>> for Circle {
    fn contains(&self, p: &geo::Point<f64>) -> bool {
        self.radius > (self.centroid.distance(p) * 100.0 * 1000.0)
    }
}

pub trait Heading {
    fn heading_degrees(&self, other: &Self) -> f64;
}

impl Heading for geo::Point<f64> {
    fn heading_degrees(&self, other: &Self) -> f64 {
        // from https://gist.github.com/jeromer/2005586
        let lat1 = self.lat().to_radians();
        let lat2 = other.lat().to_radians();
        let diff_lng = (other.lng() - self.lng()).to_radians();

        let x = diff_lng.sin() * lat2.cos();
        let y = lat1.cos() * lat2.sin() - (lat1.sin() * lat2.cos() * diff_lng.cos());

        let initial_bearing = x.atan2(y).to_degrees();
        (initial_bearing + 360.0) % 360.0
    }
}

impl Heading for TrackPoint {
    fn heading_degrees(&self, other: &Self) -> f64 {
        self.point.heading_degrees(&other.point)
    }
}
