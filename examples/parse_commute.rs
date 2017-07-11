extern crate strava_gpx;

fn main() {
    let gpx = strava_gpx::Gpx::open("examples/data/20170517-001923-Ride.gpx").unwrap();
    println!("parsed commute successfully");
}
