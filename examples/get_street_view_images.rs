extern crate chrono;
extern crate itertools;
extern crate strava_gpx;

use chrono::prelude::*;
use itertools::Itertools;

use strava_gpx::{Heading, TrackPoint, TrackPointCollection};

struct StreetViewParams {
    lat: f64,
    lng: f64,
    heading: f64,
}

fn main() {
    let gpx = strava_gpx::Gpx::open("examples/data/20170517-001923-Ride.gpx").unwrap();

    // grab every 15th track point
    let street_view_params: Vec<StreetViewParams> = gpx.track_points
        .chunks(15)
        .flat_map(|chunk| chunk.first())
        .by_ref()
        .tuple_windows()
        .map(|(tp1, tp2): (&TrackPoint, &TrackPoint)| {
            StreetViewParams {
                lat: tp1.point.lat(),
                lng: tp1.point.lng(),
                heading: tp1.heading_degrees(tp2),
            }
        })
        .collect();
}
