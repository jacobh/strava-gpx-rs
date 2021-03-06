extern crate geo;
extern crate glob;
extern crate itertools;
extern crate rayon;
extern crate strava_gpx;

use geo::contains::Contains;
use itertools::Itertools;
use rayon::prelude::*;
use strava_gpx::TrackPointCollection;


#[allow(dead_code)]
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
    let home_region = strava_gpx::Circle::new(-37.7727287, 144.9647453, 50.0);
    let work_region = strava_gpx::Circle::new(-37.8002519, 144.9846860, 50.0);

    let gpx_paths: Vec<_> = glob::glob("examples/data/all_activities/*.gpx")
        .unwrap()
        .flat_map(|entry| entry.ok())
        .collect();

    let gpxs: Vec<_> = gpx_paths
        .par_iter()
        .map(|path| strava_gpx::Gpx::open(path).unwrap())
        .collect();


    let commute_gpxs: Vec<_> = gpxs.iter()
        .filter(|gpx| {
            home_region.contains(&gpx.track_points[0].point)
                && work_region.contains(&gpx.track_points[gpx.track_points.len() - 1].point)
                && gpx.distance_meters() < 6000.0
        })
        .sorted_by(|a, b| Ord::cmp(&a.duration(), &b.duration()));

    let commute_gpxs_len = commute_gpxs.len();

    // for commute in &commute_gpxs {
    //     let avg_speed: f64 = {
    //         commute.distance_meters() / commute.duration().num_seconds() as f64 / 1000.0 * 60.0 *
    //             60.0
    //     };
    //     println!(
    //         "{}\t\t{}km\t\t{}km/h",
    //         commute.duration(),
    //         commute.distance_meters() / 1000.0,
    //         avg_speed
    //     );
    // }

    const MAX_DISTANCE_THRESHOLD: f64 = 20.0;

    let grouped_commutes: Vec<Vec<&strava_gpx::Gpx>> = commute_gpxs.iter().fold(
        vec![],
        |mut acc, &x| match acc.is_empty() {
            true => {
                acc.push(vec![x]);
                return acc;
            }
            false => {
                let mut inserted = false;
                for group in acc.iter_mut() {
                    let mean_max_distance_from_group: f64 = {
                        group
                            .iter()
                            .map(|gpx| gpx.max_distance_apart(x))
                            .sum::<f64>() / group.len() as f64
                    };
                    if mean_max_distance_from_group < MAX_DISTANCE_THRESHOLD {
                        group.push(x);
                        inserted = true;
                        break;
                    }
                }
                if !inserted {
                    acc.push(vec![x]);
                }
                return acc;
            }
        },
    );

    println!("total:          {}", gpxs.len());
    println!("commutes:       {}", commute_gpxs_len);
    println!("commute groups: {}", grouped_commutes.len());
}
