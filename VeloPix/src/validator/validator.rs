use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::event_model::hit::Hit;
use crate::event_model::track::Track;
use crate::validator::mc_particles::MCParticle;
use crate::validator::efficientcy::Efficiency;
use crate::validator::event::ValidatorEvent;

use crate::validator::helper::{
    comp_weights, hit_purity, ghost_rate, update_efficiencies, parse_montecarlo
};
/// Exposed to Python: computes and returns the reconstruction efficiency (a value in [0, 1])
/// for a given particle type from a set of event dictionaries and corresponding tracks.
#[pyfunction]
pub fn validate_efficiency(
    py_events: Vec<&PyDict>, 
    py_tracks: Vec<Vec<Track>>, 
    particle_type: &str
) -> PyResult<f64> {
    Python::with_gil(|py| {

        let mut eff: Option<Efficiency> = None;
        let mut tracking_data = Vec::new();

        // Build ValidatorEvent objects.
        for dict in py_events.iter() {
            let module_prefix_sum: Vec<usize> = dict.get_item("module_prefix_sum").unwrap().extract()?;
            let hit_xs: Vec<f64> = dict.get_item("x").unwrap().extract()?;
            let hit_ys: Vec<f64> = dict.get_item("y").unwrap().extract()?;
            let hit_zs: Vec<f64> = dict.get_item("z").unwrap().extract()?;
            let mut hits = Vec::new();
            for (i, (&x, (&y, &z))) in hit_xs.iter().zip(hit_ys.iter().zip(hit_zs.iter())).enumerate() {
                hits.push(Hit::new(x, y, z, i as i32, None, None, None));
            }
            // If "montecarlo" exists, parse it to get particles and mcp_to_hits.
            let (particles, mcp_to_hits) = if dict.contains("montecarlo")? {
                parse_montecarlo(py, dict, &hits)?
            } else {
                (Vec::new(), HashMap::new())
            };

            let event = ValidatorEvent::new(
                module_prefix_sum, 
                hit_xs, 
                hit_ys, 
                hit_zs, 
                hits, 
                Some(mcp_to_hits), 
                Some(particles)
            );
            tracking_data.push(event);
        }

        // Process each event.
        for (event, tracks) in tracking_data.iter().zip(py_tracks.iter()) {
            let weights = comp_weights(tracks, event)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;
            let cond: for<'a> fn(&'a MCParticle) -> bool = match particle_type {
                "velo" => |p: &MCParticle| p.isvelo && (p.pid.abs() != 11),
                "long" => |p: &MCParticle| p.islong && (p.pid.abs() != 11),
                "long>5GeV" => |p: &MCParticle| p.islong && p.over5 && (p.pid.abs() != 11),
                "long_strange" => |p: &MCParticle| p.islong && p.strange && (p.pid.abs() != 11),
                "long_strange>5GeV" => |p: &MCParticle| p.islong && p.over5 && p.strange && (p.pid.abs() != 11),
                "long_fromb" => |p: &MCParticle| p.islong && p.fromb && (p.pid.abs() != 11),
                "long_fromb>5GeV" => |p: &MCParticle| p.islong && p.over5 && p.fromb && (p.pid.abs() != 11),
                _ => |_| false,
            };

            eff = update_efficiencies(eff, event, tracks, &weights, particle_type, cond, true);
        }

        if let Some(eff_obj) = eff {
            Ok(eff_obj.recoeff_t / 100.0)
        } else {
            Ok(0.0)
        }
    })
}


#[pyfunction]
pub fn validate_print(py_events: Vec<&PyDict>, py_tracks: Vec<Vec<Track>>) -> PyResult<()> {
    Python::with_gil(|py|{

        let mut tracking_data = Vec::new();
        for dict in py_events.iter() {
            let module_prefix_sum: Vec<usize> = dict.get_item("module_prefix_sum").unwrap().extract()?;
            let hit_xs: Vec<f64> = dict.get_item("x").unwrap().extract()?;
            let hit_ys: Vec<f64> = dict.get_item("y").unwrap().extract()?;
            let hit_zs: Vec<f64> = dict.get_item("z").unwrap().extract()?;
            let mut hits = Vec::new();
            for (i, (&x, (&y, &z))) in hit_xs.iter().zip(hit_ys.iter().zip(hit_zs.iter())).enumerate() {
                hits.push(Hit::new(x, y, z, i as i32, None, None, None));
            }
            let (particles, mcp_to_hits) = if dict.contains("montecarlo")? {
                // parse_montecarlo -> (Vec<MCParticle>, HashMap<MCParticle, Vec<Hit>>)
                parse_montecarlo(py, dict, &hits)?
            } else {
                (Vec::new(), HashMap::new())
            };

            let event = ValidatorEvent::new(
                module_prefix_sum,
                hit_xs,
                hit_ys,
                hit_zs,
                hits,
                Some(mcp_to_hits),
                Some(particles),
            );
            tracking_data.push(event);
        }

        let mut n_tracks = 0;
        let mut n_allghosts = 0;
        let mut avg_ghost_rate = 0.0;
        let mut eff_velo: Option<Efficiency> = None;
        let mut eff_long: Option<Efficiency> = None;
        let mut eff_long5: Option<Efficiency> = None;
        let mut eff_long_strange: Option<Efficiency> = None;
        let mut eff_long_strange5: Option<Efficiency> = None;
        let mut eff_long_fromb: Option<Efficiency> = None;
        let mut eff_long_fromb5: Option<Efficiency> = None;

        // Process each event.
        for (event, tracks) in tracking_data.iter().zip(py_tracks.iter()) {
            n_tracks += tracks.len();
            let weights = comp_weights(tracks, event)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;
            if let Some(ref parts) = event.particles {
                let (t2p_map, _) = hit_purity(tracks, parts.as_slice(), &weights);
                let (grate, nghosts) = ghost_rate(&t2p_map);
                n_allghosts += nghosts;
                avg_ghost_rate += grate;
            }
            let cond_velo: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.isvelo && (p.pid.abs() != 11);
            let cond_long: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.islong && (p.pid.abs() != 11);
            let cond_long5: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.islong && p.over5 && (p.pid.abs() != 11);
            let cond_long_strange: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.islong && p.strange && (p.pid.abs() != 11);
            let cond_long_strange5: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.islong && p.over5 && p.strange && (p.pid.abs() != 11);
            let cond_long_fromb: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.islong && p.fromb && (p.pid.abs() != 11);
            let cond_long_fromb5: for<'a> fn(&'a MCParticle) -> bool =
                |p: &MCParticle| p.islong && p.over5 && p.fromb && (p.pid.abs() != 11);

            eff_velo = update_efficiencies(eff_velo, event, tracks, &weights, "velo", cond_velo, true);
            eff_long = update_efficiencies(eff_long, event, tracks, &weights, "long", cond_long, true);
            eff_long5 = update_efficiencies(eff_long5, event, tracks, &weights, "long>5GeV", cond_long5, true);
            eff_long_strange = update_efficiencies(eff_long_strange, event, tracks, &weights, "long_strange", cond_long_strange, true);
            eff_long_strange5 = update_efficiencies(eff_long_strange5, event, tracks, &weights, "long_strange>5GeV", cond_long_strange5, true);
            eff_long_fromb = update_efficiencies(eff_long_fromb, event, tracks, &weights, "long_fromb", cond_long_fromb, true);
            eff_long_fromb5 = update_efficiencies(eff_long_fromb5, event, tracks, &weights, "long_fromb>5GeV", cond_long_fromb5, true);
        }

        let nevents = tracking_data.len();
        if nevents > 0 {
            println!(
                "{} tracks including {} ghosts ({:.1}%). Event average ghost rate: {:.1}%",
                n_tracks,
                n_allghosts,
                100.0 * n_allghosts as f64 / n_tracks as f64,
                100.0 * avg_ghost_rate / nevents as f64
            );
        } else {
            println!("No events found.");
        }
        if let Some(e) = eff_velo { println!("{}", e); }
        if let Some(e) = eff_long { println!("{}", e); }
        if let Some(e) = eff_long5 { println!("{}", e); }
        if let Some(e) = eff_long_strange { println!("{}", e); }
        if let Some(e) = eff_long_strange5 { println!("{}", e); }
        if let Some(e) = eff_long_fromb { println!("{}", e); }
        if let Some(e) = eff_long_fromb5 { println!("{}", e); }

        Ok(())
    })
}

#[pyfunction]
pub fn validate_to_json(py_events: Vec<&PyDict>, py_tracks: Vec<Vec<Track>>, verbose: bool) -> PyResult<PyObject> {    
    Python::with_gil(|py|{

        let mut total_tracks = 0;
        let mut total_ghosts = 0;
        let mut total_ghost_rate = 0.0;
        let n_events = py_events.len();
        
        let cond_map: HashMap<&str, for<'a> fn(&'a MCParticle) -> bool> = [
            ("velo", (|p: &MCParticle| p.isvelo && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long", (|p: &MCParticle| p.islong && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long>5GeV", (|p: &MCParticle| p.islong && p.over5 && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_strange", (|p: &MCParticle| p.islong && p.strange && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_strange>5GeV", (|p: &MCParticle| p.islong && p.over5 && p.strange && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_fromb", (|p: &MCParticle| p.islong && p.fromb && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_fromb>5GeV", (|p: &MCParticle| p.islong && p.over5 && p.fromb && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
        ].iter().cloned().collect();
        
        let mut eff_map: HashMap<String, Efficiency> = HashMap::new();
        
        // we cache this to speedup the lookup
        let key_module_prefix_sum = "module_prefix_sum";
        let key_x = "x";
        let key_y = "y";
        let key_z = "z";
        
        for (dict, tracks) in py_events.iter().zip(py_tracks.iter()) {
            // Extract event data from the Python dictionary.
            let module_prefix_sum: Vec<usize> = dict.get_item(key_module_prefix_sum).unwrap().extract()?;
            let hit_xs: Vec<f64> = dict.get_item(key_x).unwrap().extract()?;
            let hit_ys: Vec<f64> = dict.get_item(key_y).unwrap().extract()?;
            let hit_zs: Vec<f64> = dict.get_item(key_z).unwrap().extract()?;
            
            let mut hits = Vec::with_capacity(hit_xs.len());
            for (i, (&x, (&y, &z))) in hit_xs.iter().zip(hit_ys.iter().zip(hit_zs.iter())).enumerate() {
                hits.push(Hit::new(x, y, z, i as i32, None, None, None));
            }
            
            let (particles, mcp_to_hits) = if dict.contains("montecarlo")? {
                parse_montecarlo(py, dict, &hits)? 
            } else {
                (Vec::new(), HashMap::new())
            };
            
            let event = ValidatorEvent::new(
                module_prefix_sum,
                hit_xs,
                hit_ys,
                hit_zs,
                hits,
                Some(mcp_to_hits),
                Some(particles),
            );
            
            // process the event alongside its corresponding tracks
            total_tracks += tracks.len();
            let weights = comp_weights(tracks, &event)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
            
            if let Some(ref parts) = event.particles {
                let (t2p_map, _) = hit_purity(tracks, parts.as_slice(), &weights);
                let (grate, ghosts) = ghost_rate(&t2p_map);
                total_ghosts += ghosts;
                total_ghost_rate += grate;
            }
            
            // update efficiencies
            for (cat, cond) in &cond_map {
                let key = cat.to_string();
                let current_eff = eff_map.remove(&key);
                if let Some(new_eff) = update_efficiencies(current_eff, &event, tracks, &weights, cat, *cond, verbose) {
                    eff_map.insert(key, new_eff);
                }
            }
        }
        
        let overall_ghost_rate = if total_tracks > 0 {
            100.0 * total_ghosts as f64 / total_tracks as f64
        } else {
            0.0
        };
        let event_avg_ghost_rate: f64 = if n_events > 0 {
            100.0 * total_ghost_rate / n_events as f64
        } else {
            0.0
        };
        
        let summary = PyDict::new(py);
        summary.set_item("total_tracks", total_tracks)?;
        summary.set_item("total_ghosts", total_ghosts)?;
        summary.set_item("overall_ghost_rate", overall_ghost_rate)?;
        summary.set_item("event_avg_ghost_rate", event_avg_ghost_rate)?;
        
        let mut categories_summary = Vec::new();
        for (cat, eff) in &eff_map {
            let clone_percentage = if eff.n_reco > 0 {
                100.0 * eff.n_clones as f64 / eff.n_reco as f64
            } else {
                0.0
            };
            let hit_eff_percentage = if eff.n_hits > 0 {
                100.0 * eff.n_heff / eff.n_hits as f64
            } else {
                0.0
            };
            let cat_dict = PyDict::new(py);
            cat_dict.set_item("label", cat)?;
            cat_dict.set_item("n_reco", eff.n_reco)?;
            cat_dict.set_item("n_particles", eff.n_particles)?;
            cat_dict.set_item("recoeffT", eff.recoeff_t)?;
            cat_dict.set_item("avg_recoeff", eff.avg_recoeff)?;
            cat_dict.set_item("n_clones", eff.n_clones)?;
            cat_dict.set_item("clone_percentage", clone_percentage)?;
            cat_dict.set_item("purityT", eff.purity_t)?;
            cat_dict.set_item("avg_purity", eff.avg_purity)?;
            cat_dict.set_item("avg_hiteff", eff.avg_hiteff)?;
            cat_dict.set_item("hit_eff_percentage", hit_eff_percentage)?;
            categories_summary.push(cat_dict.to_object(py));
        }
        summary.set_item("categories", categories_summary)?;
        
        Ok(summary.to_object(py))
    })
}

#[pyfunction]
pub fn validate_to_json_nested(py_events: Vec<&PyDict>, py_tracks: Vec<Vec<Track>>, verbose: bool) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let mut total_tracks = 0;
        let mut total_ghosts = 0;
        let mut total_ghost_rate = 0.0;
        let n_events = py_events.len();

        // define the category conditions.
        let cond_map: HashMap<&str, for<'a> fn(&'a MCParticle) -> bool> = [
            ("velo", (|p: &MCParticle| p.isvelo && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long", (|p: &MCParticle| p.islong && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long>5GeV", (|p: &MCParticle| p.islong && p.over5 && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_strange", (|p: &MCParticle| p.islong && p.strange && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_strange>5GeV", (|p: &MCParticle| p.islong && p.over5 && p.strange && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_fromb", (|p: &MCParticle| p.islong && p.fromb && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
            ("long_fromb>5GeV", (|p: &MCParticle| p.islong && p.over5 && p.fromb && (p.pid.abs() != 11)) as for<'a> fn(&'a MCParticle) -> bool),
        ].iter().cloned().collect();

        let mut eff_map: HashMap<String, Efficiency> = HashMap::new();
        let events_eff_obj = PyDict::new(py);

        // we cache this to speedup the lookup
        let key_module_prefix_sum = "module_prefix_sum";
        let key_x = "x";
        let key_y = "y";
        let key_z = "z";

        for (i, (dict, tracks)) in py_events.iter().zip(py_tracks.iter()).enumerate() {
            let module_prefix_sum: Vec<usize> = dict.get_item(key_module_prefix_sum).unwrap().extract()?;
            let hit_xs: Vec<f64> = dict.get_item(key_x).unwrap().extract()?;
            let hit_ys: Vec<f64> = dict.get_item(key_y).unwrap().extract()?;
            let hit_zs: Vec<f64> = dict.get_item(key_z).unwrap().extract()?;
            
            let mut hits = Vec::with_capacity(hit_xs.len());
            for (i, (&x, (&y, &z))) in hit_xs.iter().zip(hit_ys.iter().zip(hit_zs.iter())).enumerate() {
                hits.push(Hit::new(x, y, z, i as i32, None, None, None));
            }
            
            // parse Monte Carlo particles if available.
            let (particles, mcp_to_hits) = if dict.contains("montecarlo")? {
                parse_montecarlo(py, dict, &hits)?
            } else {
                (Vec::new(), HashMap::new())
            };
            
            let event = ValidatorEvent::new(
                module_prefix_sum,
                hit_xs,
                hit_ys,
                hit_zs,
                hits,
                Some(mcp_to_hits),
                Some(particles),
            );
            
            // process the event alongside its corresponding tracks.
            total_tracks += tracks.len();
            let weights = comp_weights(tracks, &event)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
            
            if let Some(ref parts) = event.particles {
                let (t2p_map, _) = hit_purity(tracks, parts.as_slice(), &weights);
                let (grate, ghosts) = ghost_rate(&t2p_map);
                total_ghosts += ghosts;
                total_ghost_rate += grate;
            }
            
            // build a list of efficiency dictionaries for this event
            let mut event_eff_list = Vec::new();
            for (cat, cond) in &cond_map {
                let key = cat.to_string();
                let current_eff = eff_map.remove(&key);
                if let Some(new_eff) = update_efficiencies(current_eff, &event, tracks, &weights, cat, *cond, verbose) {
                    eff_map.insert(key.clone(), new_eff.clone());
                    let cat_eff_dict = PyDict::new(py);
                    cat_eff_dict.set_item("label", key.clone())?;
                    cat_eff_dict.set_item("n_reco", new_eff.n_reco)?;
                    cat_eff_dict.set_item("n_particles", new_eff.n_particles)?;
                    cat_eff_dict.set_item("recoeffT", new_eff.recoeff_t)?;
                    cat_eff_dict.set_item("avg_recoeff", new_eff.avg_recoeff)?;
                    cat_eff_dict.set_item("n_clones", new_eff.n_clones)?;
                    let clone_percentage = if new_eff.n_reco > 0 {
                        100.0 * new_eff.n_clones as f64 / new_eff.n_reco as f64
                    } else {
                        0.0
                    };
                    cat_eff_dict.set_item("clone_percentage", clone_percentage)?;
                    cat_eff_dict.set_item("purityT", new_eff.purity_t)?;
                    cat_eff_dict.set_item("avg_purity", new_eff.avg_purity)?;
                    cat_eff_dict.set_item("avg_hiteff", new_eff.avg_hiteff)?;
                    let hit_eff_percentage = if new_eff.n_hits > 0 {
                        100.0 * new_eff.n_heff as f64 / new_eff.n_hits as f64
                    } else {
                        0.0
                    };
                    cat_eff_dict.set_item("hit_eff_percentage", hit_eff_percentage)?;
                    event_eff_list.push(cat_eff_dict.to_object(py));
                }
            }
            // slight inconviniance giving this the event name but rather complicate stuff in how I handle the JSON object in python than here so there u go        
            let event_key: String = (i + 1).to_string();
            events_eff_obj.set_item(event_key, event_eff_list)?;
        }
        
        let overall_ghost_rate = if total_tracks > 0 {
            100.0 * total_ghosts as f64 / total_tracks as f64
        } else {
            0.0
        };
        let event_avg_ghost_rate: f64 = if n_events > 0 {
            100.0 * total_ghost_rate / n_events as f64
        } else {
            0.0
        };

        let summary = PyDict::new(py);
        summary.set_item("total_tracks", total_tracks)?;
        summary.set_item("total_ghosts", total_ghosts)?;
        summary.set_item("overall_ghost_rate", overall_ghost_rate)?;
        summary.set_item("event_avg_ghost_rate", event_avg_ghost_rate)?;
        
        let mut categories_summary = Vec::new();
        for (cat, eff) in &eff_map {
            let clone_percentage = if eff.n_reco > 0 {
                100.0 * eff.n_clones as f64 / eff.n_reco as f64
            } else {
                0.0
            };
            let hit_eff_percentage = if eff.n_hits > 0 {
                100.0 * eff.n_heff as f64 / eff.n_hits as f64
            } else {
                0.0
            };
            let cat_dict = PyDict::new(py);
            cat_dict.set_item("label", cat)?;
            cat_dict.set_item("n_reco", eff.n_reco)?;
            cat_dict.set_item("n_particles", eff.n_particles)?;
            cat_dict.set_item("recoeffT", eff.recoeff_t)?;
            cat_dict.set_item("avg_recoeff", eff.avg_recoeff)?;
            cat_dict.set_item("n_clones", eff.n_clones)?;
            cat_dict.set_item("clone_percentage", clone_percentage)?;
            cat_dict.set_item("purityT", eff.purity_t)?;
            cat_dict.set_item("avg_purity", eff.avg_purity)?;
            cat_dict.set_item("avg_hiteff", eff.avg_hiteff)?;
            cat_dict.set_item("hit_eff_percentage", hit_eff_percentage)?;
            categories_summary.push(cat_dict.to_object(py));
        }
        summary.set_item("categories", categories_summary)?;
        summary.set_item("events", events_eff_obj)?;
        
        Ok(summary.to_object(py))
    })
}
