use pyo3::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use crate::event_model::event::Event;
use crate::event_model::hit::Hit;
use crate::event_model::track::Track;
use crate::event_model::module::Module;

#[pyclass]
#[derive(Clone)]
#[pyo3(text_signature = "(cls, max_scatter=None, min_track_length=None, min_strong_track_length=None, allowed_missed_modules=None)")]
pub struct SearchByTripletTrie {
    max_scatter: f64,
    min_track_length: usize,
    min_strong_track_length: usize,
    allowed_missed_modules: usize, // still used logically via the miss flags below
}

#[pymethods]
impl SearchByTripletTrie {
    #[new]
    pub fn new(
        max_scatter: Option<f64>,
        min_track_length: Option<usize>,
        min_strong_track_length: Option<usize>,
        allowed_missed_modules: Option<usize>,
    ) -> Self {
        let max_scatter = max_scatter.unwrap_or(0.1);
        let min_track_length = min_track_length.unwrap_or(3);
        let min_strong_track_length = min_strong_track_length.unwrap_or(4);
        let allowed_missed_modules = allowed_missed_modules.unwrap_or(2);

        println!("Instantiating TrackScatterSolver with parameters:");
        println!("  max_scatter: {}", max_scatter);
        println!("  min_track_length: {}", min_track_length);
        println!("  min_strong_track_length: {}", min_strong_track_length);
        println!("  allowed_missed_modules: {}", allowed_missed_modules);

        SearchByTripletTrie {
            max_scatter,
            min_track_length,
            min_strong_track_length,
            allowed_missed_modules,
        }
    }

    /// Public method exposed to Python.
    /// Processes the event to assemble tracks.
    #[pyo3(text_signature = "($self, event)")]
    pub fn solve(&self, event: &Event) -> Vec<Track> {
        let module_pairs = self.merge_module_pairs(event);
        let compatible_triplets_trie = self.generate_compatible_triplets(&module_pairs);
    
        let mut flagged_hits: HashSet<Hit> = HashSet::new();
        let mut forwarding_tracks: Vec<Track> = Vec::new();
        let mut final_tracks: Vec<Track> = Vec::new();
        let mut weak_tracks: Vec<Track> = Vec::new();
    
        if module_pairs.len() < 3 {
            return final_tracks;
        }
    
        // Use reversed slices similar to the Python version.
        let slice_m0 = &module_pairs[2..];
        let slice_m1 = &module_pairs[..(module_pairs.len() - 2)];
        for (m0, m1) in slice_m0.iter().rev().zip(slice_m1.iter().rev()) {
            let compatible_triplets_in_module = if (m0.module_number as usize) < compatible_triplets_trie.len() {
                compatible_triplets_trie[m0.module_number as usize].as_ref()
            } else {
                None
            };
    
            let mut forwarding_next_step: Vec<Track> = Vec::new();
    
            // Process each track from the previous iteration.
            for mut t in forwarding_tracks {
                // Ensure the track has at least 2 hits (seed length).
                if t.hits.len() < 2 {
                    continue;
                }
                // Extract required values before any modification.
                let len_hits = t.hits.len();
                let h0 = t.hits[len_hits - 2].clone();
                let h1 = t.hits[len_hits - 1].clone();
                let prev_missed = t.missed_last_module; // bool is Copy
    
                // Branch 1: Extend via precomputed compatible triplets.
                if let Some(comp_module) = compatible_triplets_in_module {
                    if let Some(inner) = comp_module.get(&h0) {
                        if let Some((h2, _scatter)) = inner.get(&h1) {
                            t.hits.push(h2.clone());
                            flagged_hits.insert(h2.clone());
                            if t.hits.len() >= self.min_strong_track_length {
                                for hit in t.hits.iter().take(self.min_strong_track_length - 1) {
                                    flagged_hits.insert(hit.clone());
                                }
                            }
                            t.missed_penultimate_module = prev_missed;
                            t.missed_last_module = false;
                            forwarding_next_step.push(t);
                            continue; // Done with this track.
                        }
                    }
                }
    
                // Branch 2: Look for an extension directly in the corresponding event module.
                let branch2_result: Option<Track> = {
                    // Clone t so that we do not move the original.
                    let mut t_branch2 = t.clone();
                    let m_index = if m1.module_number > 0 {
                        (m1.module_number - 1) as usize
                    } else {
                        0
                    };
                    if let Some(module) = event.modules.get(m_index) {
                        if let Ok(hits) = module.hits() {
                            let mut best_h2 = Hit::new(0.0, 0.0, 0.0, -1, Some(-1), Some(0.0), Some(false));
                            let mut best_scatter = self.max_scatter;
                            for h2 in hits.iter() {
                                let scatter = self.calculate_scatter(&h0, &h1, h2);
                                if !flagged_hits.contains(h2) && scatter < best_scatter {
                                    best_h2 = h2.clone();
                                    best_scatter = scatter;
                                }
                            }
                            if best_h2.id != -1 {
                                t_branch2.hits.push(best_h2.clone());
                                flagged_hits.insert(best_h2.clone());
                                if t_branch2.hits.len() >= self.min_strong_track_length {
                                    for hit in t_branch2.hits.iter().take(self.min_strong_track_length - 1) {
                                        flagged_hits.insert(hit.clone());
                                    }
                                }
                                t_branch2.missed_penultimate_module = prev_missed;
                                t_branch2.missed_last_module = false;
                                Some(t_branch2)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
    
                if let Some(track) = branch2_result {
                    forwarding_next_step.push(track);
                    continue;
                }
    
                // Branch 3: No extension found, update miss flags.
                let current_hits_len = t.hits.len();
                // Clone t to update its miss flags.
                let mut new_t = t.clone();
                new_t.missed_penultimate_module = prev_missed;
                new_t.missed_last_module = true;
                if prev_missed {
                    if current_hits_len >= self.min_strong_track_length {
                        final_tracks.push(new_t);
                    } else {
                        weak_tracks.push(new_t);
                    }
                } else {
                    forwarding_next_step.push(new_t);
                }
            }
            forwarding_tracks = forwarding_next_step;
    
            // Seeding: Create new tracks from the current module's compatible triplets.
            if let Some(comp_module) = compatible_triplets_in_module {
                for (h0, inner_map) in comp_module {
                    let mut best_h1 = Hit::new(0.0, 0.0, 0.0, -1, Some(-1), Some(0.0), Some(false));
                    let mut best_h2 = Hit::new(0.0, 0.0, 0.0, -1, Some(-1), Some(0.0), Some(false));
                    let mut best_scatter = self.max_scatter;
                    for (h1, &(ref h2, scatter)) in inner_map {
                        if !flagged_hits.contains(h0)
                            && !flagged_hits.contains(h1)
                            && !flagged_hits.contains(h2)
                            && scatter < best_scatter
                        {
                            best_scatter = scatter;
                            best_h1 = h1.clone();
                            best_h2 = h2.clone();
                        }
                    }
                    if best_scatter < self.max_scatter {
                        let new_track = Track {
                            hits: vec![h0.clone(), best_h1, best_h2],
                            missed_last_module: false,
                            missed_penultimate_module: false,
                        };
                        forwarding_tracks.push(new_track);
                    }
                }
            }
        }
    
        // Consolidate tracks.
        for t in forwarding_tracks.into_iter() {
            if t.hits.len() >= self.min_strong_track_length {
                final_tracks.push(t);
            } else {
                weak_tracks.push(t);
            }
        }
        // Process weak tracks: add them if none of their first three hits are flagged.
        for t in weak_tracks.into_iter() {
            if !flagged_hits.contains(&t.hits[0])
                && !flagged_hits.contains(&t.hits[1])
                && !flagged_hits.contains(&t.hits[2])
            {
                final_tracks.push(t);
            }
        }
    
        final_tracks
    }         

    // #[pyo3(text_signature = "($self, events)")]
    // pub fn solve_parallel(&self, events: Vec<Event>) -> PyResult<Vec<Vec<Track>>> {
    //     let results: Vec<PyResult<Vec<Track>>> = events
    //         .into_par_iter()
    //         .map(|event| self.solve(&event))
    //         .collect();

    //     let mut all_tracks_per_event = Vec::with_capacity(results.len());
    //     for event_tracks in results {
    //         all_tracks_per_event.push(event_tracks?);
    //     }
    //     Ok(all_tracks_per_event)
    // }
}

// Private helper methods not exposed to Python.
impl SearchByTripletTrie {
    fn merge_module_pairs(&self, ev: &Event) -> Vec<Module> {
        let mut module_pairs = Vec::new();
        let modules = &ev.modules;
        for pair in modules.chunks(2) {
            if pair.len() < 2 {
                continue;
            }
            let m0 = &pair[0];
            let m1 = &pair[1];
            let merged_module_number = m0.module_number / 2;
            let hit_start_index = m0.hit_start_index;
            let hit_length = m1.hit_end_index.saturating_sub(m0.hit_start_index);
            let merged_module = Module {
                module_number: merged_module_number,
                z: m0.z,
                hit_start_index,
                hit_end_index: hit_start_index + hit_length,
                global_hits: m0.global_hits.clone(),
            };
            module_pairs.push(merged_module);
        }
        module_pairs
    }

    fn calculate_scatter(&self, h0: &Hit, h1: &Hit, h2: &Hit) -> f64 {
        let td = 1.0 / (h1.z - h0.z);
        let txn = h1.x - h0.x;
        let tyn = h1.y - h0.y;
        let tx = txn * td;
        let ty = tyn * td;
        let dz = h2.z - h0.z;
        let x_prediction = h0.x + tx * dz;
        let y_prediction = h0.y + ty * dz;
        let dx = x_prediction - h2.x;
        let dy = y_prediction - h2.y;
        dx * dx + dy * dy
    }

    fn check_best_triplets(
        &self,
        m0: &Module,
        m1: &Module,
        m2: &Module,
    ) -> Vec<(Hit, Hit, Hit, f64)> {
        let mut best_triplets = Vec::new();
        let hits0 = m0.hits().unwrap();
        let hits1 = m1.hits().unwrap();
        let hits2 = m2.hits().unwrap();
    
        for h0 in hits0.iter() {
            for h1 in hits1.iter() {
                let mut best_h2 = Hit::new(0.0, 0.0, 0.0, -1, Some(-1), Some(0.0), Some(false));
                let mut best_scatter = self.max_scatter;
                for h2 in hits2.iter() {
                    let scatter = self.calculate_scatter(h0, h1, h2);
                    if scatter < best_scatter {
                        best_h2 = h2.clone();
                        best_scatter = scatter;
                    }
                }
                if best_scatter < self.max_scatter {
                    best_triplets.push((h0.clone(), h1.clone(), best_h2, best_scatter));
                }
            }
        }
        best_triplets
    }

    fn generate_compatible_triplets(
        &self,
        module_pairs: &[Module],
    ) -> Vec<Option<HashMap<Hit, HashMap<Hit, (Hit, f64)>>>> {
        let mut compatible_triplets_trie = vec![None; 26];
        if module_pairs.len() < 3 {
            return compatible_triplets_trie;
        }
        let slice_m0 = &module_pairs[2..];
        let slice_m1 = &module_pairs[1..(module_pairs.len() - 1)];
        for (m0, m1) in slice_m0.iter().rev().zip(slice_m1.iter().rev()) {
            let mut compatible_triplets_module: HashMap<Hit, HashMap<Hit, (Hit, f64)>> =
                HashMap::new();
            let m2_index = if m1.module_number > 0 {
                (m1.module_number - 1) as usize
            } else {
                0
            };
            if m2_index >= module_pairs.len() {
                continue;
            }
            let best_triplets = self.check_best_triplets(m0, m1, &module_pairs[m2_index]);
            for (h0, h1, h2, scatter) in best_triplets {
                compatible_triplets_module
                    .entry(h0.clone())
                    .or_insert_with(HashMap::new)
                    .insert(h1.clone(), (h2, scatter));
            }
            let idx = m0.module_number as usize;
            if idx < compatible_triplets_trie.len() {
                compatible_triplets_trie[idx] = Some(compatible_triplets_module);
            }
        }
        compatible_triplets_trie
    }
}
