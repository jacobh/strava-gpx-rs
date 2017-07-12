extern crate strava_gpx;

fn main() {
    let gpx = strava_gpx::Gpx::open("examples/data/20170517-001923-Ride.gpx").unwrap();
    println!("parsed commute successfully");
    println!("track points: {}", gpx.track_points.len());
    println!("distance:     {}", gpx.distance_meters());
    for speed in gpx.speed_meters_per_sec() {
        println!("{}", speed);
    }
}
