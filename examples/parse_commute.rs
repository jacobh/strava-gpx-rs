extern crate strava_gpx;
use strava_gpx::TrackPointCollection;

fn main() {
    let gpx = strava_gpx::Gpx::open("examples/data/20170517-001923-Ride.gpx").unwrap();
    println!("parsed commute successfully");
    println!("track points:   {}", gpx.track_points.len());
    println!("distance:       {}", gpx.distance_meters());
    println!("elevation gain: {}", gpx.total_elevation_gain_meters());
    for speed in gpx.as_speed_meters_per_sec() {
        println!("{}", speed);
    }
}
