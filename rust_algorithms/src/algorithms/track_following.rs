use pyo3::prelude::*;
use crate::event_model::hit::Hit;
use crate::event_model::event::Event;
use crate::event_model::module::Module;
use crate::event_model::track::Track;

#[pyclass]
#[derive(Clone)]
#[pyo3(text_signature = "(cls, max_slopes=None, max_tolerance=None, max_scatter=None, min_track_length=None, min_strong_track_length=None)")]
pub struct TrackFollowing {
    max_slopes: (f64, f64),
    max_tolerance: (f64, f64),
    max_scatter: f64,
    min_track_length: u8,
    min_strong_track_length: u8
}

#[pymethods]
impl TrackFollowing {
    #[new]
    pub fn new(
        max_slopes: Option<(f64, f64)>,
        max_tolerance: Option<(f64, f64)>,
        max_scatter: Option<f64>,
        min_track_length: Option<u8>,
        min_strong_track_length: Option<u8>
    ) -> Self {
        let max_slopes: (f64, f64) = max_slopes.unwrap_or((0.7, 0.7));
        let max_tolerance: (f64, f64) = max_tolerance.unwrap_or((0.4, 0.4));
        let max_scatter: f64 = max_scatter.unwrap_or(0.4);
        let min_track_length: u8 = min_track_length.unwrap_or(3);
        let min_strong_track_length: u8 = min_strong_track_length.unwrap_or(4);


        println!(
            "Instantiating track_following solver with parameters\n max slopes: {:?}\n max tolerance: {:?}\n max scatter: {}\n",
            max_slopes, max_tolerance, max_scatter
        );

        TrackFollowing {
            max_slopes,
            max_tolerance,
            max_scatter,
            min_track_length,
            min_strong_track_length
        }
    }

    // Returns true if two hits are compatible based on the slope criteria.
    pub fn are_compatible(&self, hit0: &Hit, hit1: &Hit) -> PyResult<bool> {
        let hit_distance: f64 = (hit0.z - hit1.z).abs();
        let dx_max: f64 = self.max_slopes.0 * hit_distance;
        let dy_max: f64 = self.max_slopes.1 * hit_distance;
        Ok((hit1.x - hit0.x).abs() < dx_max && (hit1.y - hit0.y).abs() < dy_max)
    }

    // Checks if three hits are in tolerance.
    pub fn check_tolerance(&self, hit0: &Hit, hit1: &Hit, hit2: &Hit) -> PyResult<bool> {
        let dz01: f64 = hit1.z - hit0.z;
        if dz01 == 0.0 {
            return Ok(false);
        }
        let td: f64 = 1.0 / dz01;
        let txn: f64 = hit1.x - hit0.x;
        let tyn: f64 = hit1.y - hit0.y;
        let tx: f64 = txn * td;
        let ty: f64 = tyn * td;

        let dz: f64 = hit2.z - hit0.z;
        let x_prediction: f64 = hit0.x + tx * dz;
        let dx: f64 = (x_prediction - hit2.x).abs();

        let y_prediction: f64 = hit0.y + ty * dz;
        let dy: f64 = (y_prediction - hit2.y).abs();
        
        let scatter_num = (dx * dy) + (dy * dy);
        let dz21: f64 = hit2.z - hit1.z;
        if dz21 == 0.0 {
            return Ok(false);
        }
        let scatter_denom: f64 = 1.0 / dz21;
        Ok(
            dx < self.max_tolerance.0 
            && dy < self.max_tolerance.1 
            && scatter_num * scatter_denom * scatter_denom < self.max_scatter
        )
    }

    // Given an Event, attempts to solve and find tracks.
    // Returns a list of Tracks.
    #[pyo3(text_signature = "($self, event)")]
    pub fn solve(&self, event: &Event) -> PyResult<Vec<Track>> {
        let mut weak_tracks: Vec<Track> = Vec::new();
        let mut tracks: Vec<Track> = Vec::new();
        let mut used_hits: Vec<Hit> = Vec::new();

        let modules = &event.modules;
        let num_modules = modules.len();
        if num_modules < 3 {
            return Ok(Vec::new());
        }

        // Create seed candidates by zipping reversed slices.
        let s0_rev: Vec<&Module> = modules[3..].iter().rev().collect();
        let s1_rev: Vec<&Module> = modules[1..(num_modules - 2)].iter().rev().collect();
        let indices_rev: Vec<usize> = (0..(num_modules - 3)).rev().collect();

        for ((s0, s1), &starting_module_index) in s0_rev.iter().zip(s1_rev.iter()).zip(indices_rev.iter()) {
            // For each candidate hit in module s0 not yet used.
            for h0 in s0.hits()? {
                if used_hits.contains(&h0) {
                    continue;
                }
                for h1 in s1.hits()? {
                    if used_hits.contains(&h1) {
                        continue;
                    } 
                    if !self.are_compatible(&h0, &h1)? {
                        continue;
                    }
                    // We have a seed: h0 and h1 are compatible.
                    let mut forming_track = Track::new(vec![h0.clone(), h1.clone()]);
                    let mut h2_found: bool = false;
                    let mut module_index_iter: isize = -1;

                    // Search for a third hit (h2) in modules from (starting_module_index - 2) to starting_module_index.
                    let lower = if starting_module_index >= 2 {
                        starting_module_index - 2
                    } else {
                        0
                    };
                    let range: Vec<usize> = (lower..=starting_module_index).collect();
                    let range_rev: Vec<usize> = range.into_iter().rev().collect();
                    for module_index in range_rev {
                        if module_index >= modules.len() {
                            continue;
                        }
                        let module = &modules[module_index];
                        for h2 in module.hits()? {
                            if self.check_tolerance(&h0, &h1, &h2)? {
                                forming_track.add_hit(h2.clone());
                                h2_found = true;
                                module_index_iter = module_index as isize;
                                break; 
                            }
                        }
                        if h2_found {
                            break;
                        }
                    }

                    if h2_found {
                        // Continue "forward" with the track.
                        let mut missing_stations = 0;
                        while module_index_iter > 0 && missing_stations < 3 {
                            module_index_iter -= 1;
                            missing_stations += 1;
                            let module = &modules[module_index_iter as usize];
                            let track_hits = &forming_track.hits;
                            if track_hits.len() < 2 {
                                break;
                            }
                            let last_but_one = &track_hits[track_hits.len() - 2];
                            let last = &track_hits[track_hits.len() - 1];
                            for h2 in module.hits()? {
                                if self.check_tolerance(last_but_one, last, &h2)? {
                                    forming_track.add_hit(h2.clone());
                                    missing_stations = 0;
                                    break;
                                }
                            }
                        }
                        // Classify track as weak or strong.
                        if forming_track.hits.len() == self.min_track_length.into() {
                            weak_tracks.push(forming_track);
                        } else if forming_track.hits.len() >= self.min_strong_track_length.into() {
                            tracks.push(forming_track.clone());
                            // Mark hits as used.
                            for h in forming_track.hits.iter() {
                                if !used_hits.contains(h) {
                                    used_hits.push(h.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        // Process weak tracks: add them if none of their hits are already used.
        for t in weak_tracks {
            let mut used_in_weak: bool = false;
            for h in t.hits.iter() {
                if used_hits.contains(h) {
                    used_in_weak = true;
                    break;
                }
            }
            if !used_in_weak {
                for h in t.hits.iter() {
                    if !used_hits.contains(h) {
                        used_hits.push(h.clone());
                    }
                }
                tracks.push(t);
            }
        }
        Ok(tracks)
    }
}
