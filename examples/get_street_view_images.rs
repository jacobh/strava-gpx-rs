extern crate chrono;
extern crate dotenv;
extern crate itertools;
extern crate rayon;
extern crate reqwest;
extern crate strava_gpx;

use std::fs::File;
use chrono::prelude::*;
use itertools::Itertools;
use rayon::prelude::*;
use dotenv::dotenv;

use strava_gpx::{Heading, TrackPoint, TrackPointCollection};

struct StreetViewParams {
    lat: f64,
    lng: f64,
    heading: f64,
}

fn main() {
    dotenv().ok();

    let gpx = strava_gpx::Gpx::open("examples/data/20170517-001923-Ride.gpx").unwrap();
    let client = reqwest::Client::new();
    let GOOGLE_MAPS_API_KEY =
        ::std::env::var("GOOGLE_MAPS_API_KEY").expect("GOOGLE_MAPS_API_KEY must be set");

    // grab every 15th track point
    let points: Vec<StreetViewParams> = gpx.track_points
        .chunks(5)
        .flat_map(|chunk| chunk.first())
        .by_ref()
        .tuple_windows()
        .map(|(tp1, tp2): (&TrackPoint, &TrackPoint)| {
            println!("{}, {}", tp1.point.x(), tp1.point.y());
            StreetViewParams {
                lat: tp1.point.x(),
                lng: tp1.point.y(),
                heading: tp1.heading_degrees(tp2),
            }
        })
        .collect();

    points.par_iter().enumerate().for_each(|(i, point)| {
        let url = reqwest::Url::parse_with_params(
            "https://maps.googleapis.com/maps/api/streetview",
            &[
                ("location", format!("{},{}", point.lat, point.lng)),
                ("heading", (point.heading as i32).to_string()),
                ("size", "640x640".to_owned()),
                ("fov", "120".to_owned()),
                ("key", GOOGLE_MAPS_API_KEY.clone()),
            ],
        ).unwrap();
        println!("{}", url);
        let mut resp = client.get(url).send().unwrap();
        let mut f = File::create(format!("{}.jpg", i)).unwrap();
        std::io::copy(&mut resp, &mut f).unwrap();
    });
}
