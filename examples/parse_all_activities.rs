extern crate strava_gpx;
extern crate glob;
extern crate rayon;

use rayon::prelude::*;
use strava_gpx::TrackPointCollection;


#![allow(dead_code)]
fn pause() {
    use std::io;
    use std::io::prelude::*;

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {
    let gpx_paths: Vec<_> = glob::glob("examples/data/all_activities/*.gpx")
        .unwrap()
        .flat_map(|entry| entry.ok())
        .collect();

    let gpxs: Vec<_> = gpx_paths
        .par_iter()
        .map(|path| strava_gpx::Gpx::open(path).unwrap())
        .collect();

    let distances: Vec<_> = gpxs.par_iter().map(|gpx| gpx.distance_meters()).collect();

    for distance in distances {
        println!("km: {}", distance / 1000.0)
    }
}
