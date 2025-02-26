use std::collections::HashMap;
use std::hash::Hash;

use pyo3::types::PyDict;
use pyo3::prelude::*;

use crate::event_model::track::Track;
use crate::event_model::hit::Hit;
use crate::validator::efficientcy::Efficiency;
use crate::validator::mc_particles::MCParticle;
use crate::validator::event::ValidatorEvent;

/// Compute the weight matrix w(t, p).
/// For each track and each particle, the weight is the fraction of hits on the track that are attributed to that particle.
///
/// # Arguments
/// * `tracks` - A slice of reconstructed tracks.
/// * `event` - The event data which must contain a `particles` field and optionally a `hit_to_mcp` mapping.
pub fn comp_weights(
    tracks: &[Track],
    event: &ValidatorEvent,
) -> Result<Vec<Vec<f64>>, String> {
    let nparticles = event.particles.as_ref().map(|v| v.len()).unwrap_or(0);
    let mut weights = vec![vec![0.0; nparticles]; tracks.len()];
    if nparticles == 0 {
        return Ok(weights);
    }
    // Ensure the hit_to_mcp mapping is present.
    let hit_to_mcp = event.hit_to_mcp.as_ref().ok_or("Error: hit_to_mcp mapping is missing")?;
    
    // Precompute a mapping from hit id to its associated MCParticles.
    let mut hit_map: HashMap<i32, &Vec<MCParticle>> = HashMap::new();
    for (hit, mcps) in hit_to_mcp.iter() {
        hit_map.insert(hit.id, mcps);
    }
    
    // For each track, compute the fraction of hits associated with each particle.
    for (i, track) in tracks.iter().enumerate() {
        let track_hits = &track.hits;
        let nhits = track_hits.len();
        if nhits < 2 {
            continue;
        }
        // For each particle, count how many hits in the track are associated with it.
        for (j, particle) in event.particles.as_ref().unwrap().iter().enumerate() {
            let mut nhits_from_p = 0;
            for h in track_hits {
                if let Some(mcps) = hit_map.get(&h.id) {
                    if mcps.iter().any(|p| p == particle) {
                        nhits_from_p += 1;
                    }
                } else {
                    return Err(format!("Error: hit with id {} not found in hit_to_mcp mapping", h.id));
                }
            }
            weights[i][j] = nhits_from_p as f64 / nhits as f64;
        }
    }
    Ok(weights)
}

/// Compute hit purity maps for tracks and particles.
/// Returns two hash maps:
///  - t2p: mapping from each track to a tuple (max_weight, Option<MCParticle>).
///  - p2t: mapping from each MCParticle to a tuple (max_weight, Option<Track>).
///
/// A track is associated with a particle only if the maximum weight in its row is greater than 0.7.
pub fn hit_purity(
    tracks: &[Track],
    particles: &[MCParticle],
    weights: &[Vec<f64>],
) -> (
    HashMap<Track, (f64, Option<MCParticle>)>,
    HashMap<MCParticle, (f64, Option<Track>)>,
)
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut t2p: HashMap<Track, (f64, Option<MCParticle>)> = HashMap::new();
    let mut p2t: HashMap<MCParticle, (f64, Option<Track>)> = HashMap::new();

    // For each track, find the maximum weight and corresponding particle.
    for (i, track) in tracks.iter().enumerate() {
        let row = &weights[i];
        if let Some((max_idx, &max_w)) = row
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        {
            if max_w > 0.7 {
                t2p.insert(track.clone(), (max_w, Some(particles[max_idx].clone())));
            } else {
                t2p.insert(track.clone(), (max_w, None));
            }
        }
    }

    // For each particle, iterate over tracks to get the maximum weight for that particle.
    for (j, particle) in particles.iter().enumerate() {
        let mut max_w = 0.0;
        let mut max_track: Option<Track> = None;
        for (i, _track) in tracks.iter().enumerate() {
            let w = weights[i][j];
            if w > max_w {
                max_w = w;
                max_track = Some(tracks[i].clone());
            }
        }
        if max_w > 0.7 {
            p2t.insert(particle.clone(), (max_w, max_track));
        } else {
            p2t.insert(particle.clone(), (max_w, None));
        }
    }

    (t2p, p2t)
}

/// Return the ghost rate and ghost count given a track-to-particle mapping.
/// Ghost tracks are those with no associated particle.
pub fn ghost_rate(
    t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
) -> (f64, usize) {
    let total = t2p.len();
    let ghosts = t2p.values().filter(|&&(_w, ref opt)| opt.is_none()).count();
    let rate = if total > 0 {
        ghosts as f64 / total as f64
    } else {
        0.0
    };
    (rate, ghosts)
}

/// Returns a vector of tracks that are considered reconstructed.
/// A track is reconstructed if it has an associated particle.
pub fn reconstructed(
    p2t: &HashMap<MCParticle, (f64, Option<Track>)>,
) -> Vec<Track> {
    p2t.values()
        .filter_map(|&(_w, ref t_opt)| t_opt.clone())
        .collect()
}

/// Returns a HashMap mapping each MCParticle (that has more than one associated track)
/// to the list of tracks associated with it.
pub fn clones(
    t2p: HashMap<Track, (f64, Option<MCParticle>)>,
) -> HashMap<MCParticle, Vec<Track>>
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut particle_to_tracks: HashMap<MCParticle, Vec<Track>> = HashMap::new();
    for (track, (_w, opt_particle)) in t2p.into_iter() {
        if let Some(particle) = opt_particle {
            particle_to_tracks
                .entry(particle)
                .or_insert_with(Vec::new)
                .push(track);
        }
    }
    // Only keep particles with more than one associated track (i.e. clones).
    particle_to_tracks
        .into_iter()
        .filter(|(_, tracks)| tracks.len() > 1)
        .collect()
}

/// Compute hit efficiency for tracks that are associated with a particle.
/// The efficiency is defined as the ratio of hits on the track from the associated particle
/// to the total hits from that particle (as given by the event's mcp_to_hits map).
pub fn hit_efficiency(
    t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
    event: &ValidatorEvent,
) -> HashMap<Track, f64> {
    let mut hit_eff: HashMap<Track, f64> = HashMap::new();

    // Ensure we have the necessary mappings from the event.
    if let (Some(ref hit_to_mcp), Some(ref mcp_to_hits)) = (&event.hit_to_mcp, &event.mcp_to_hits) {
        for (track, &(_w, ref opt_particle)) in t2p.iter() {
            if let Some(ref particle) = opt_particle {
                // Count how many hits on the track are from the associated particle.
                let hits_from_particle = track.hits.iter().filter(|hit| {
                    hit_to_mcp.get(*hit)
                        .map(|mcps| mcps.iter().any(|p| p == particle))
                        .unwrap_or(false)
                }).count();

                // Total number of hits contributed by the particle (from mcp_to_hits).
                let total_hits = mcp_to_hits.get(particle)
                    .map(|v| v.len())
                    .unwrap_or(0);
                if total_hits > 0 {
                    hit_eff.insert(track.clone(), hits_from_particle as f64 / total_hits as f64);
                }
            }
        }
    }
    hit_eff
}

/// Update efficiencies for a given event and set of tracks.
///
/// This mirrors the Python logic by:
///  - Filtering the particles based on `cond`
///  - Slicing the weights matrix to only include the filtered particles
///  - Computing the hit purity maps
///  - Updating an existing Efficiency instance or creating a new one.
///
/// # Arguments
/// * `eff` - Optional current Efficiency instance.
/// * `event` - The event data.
/// * `tracks` - The list of tracks for this event.
/// * `weights` - The full weight matrix (each row corresponds to a track, each column to a particle).
/// * `label` - A label for this Efficiency instance.
/// * `cond` - A function that returns true for particles that should be considered.
pub fn update_efficiencies(
    mut eff: Option<Efficiency>,
    event: &ValidatorEvent,
    tracks: &[Track],
    weights: &[Vec<f64>],
    label: &str,
    cond: for<'a> fn(&'a MCParticle) -> bool,
) -> Option<Efficiency>
where
    Track: Clone + std::hash::Hash + Eq,
    MCParticle: Clone + std::hash::Hash + Eq,
{
    let binding = vec![];
    let particles_full = event.particles.as_ref().unwrap_or(&binding);
    let filtered: Vec<(usize, MCParticle)> = particles_full
        .iter()
        .enumerate()
        .filter_map(|(i, p)| if cond(p) { Some((i, p.clone())) } else { None })
        .collect();
    
    if filtered.is_empty() {
        println!("update_efficiencies: No particles match condition for '{}'", label);
        return eff;
    }
    
    let (pidx_filtered, particles_filtered): (Vec<usize>, Vec<MCParticle>) = filtered.into_iter().unzip();

    let weights_filtered: Vec<Vec<f64>> = weights
        .iter()
        .map(|row| pidx_filtered.iter().map(|&j| row[j]).collect())
        .collect();

    let (t2p_map, p2t_map) = hit_purity(tracks, &particles_filtered, &weights_filtered);

    if let Some(ref mut e) = eff {
        e.add_event_internal(&t2p_map, &p2t_map, particles_filtered, event);
    } else {
        let mut new_eff = Efficiency::new(label.to_string());
        new_eff.add_event_internal(&t2p_map, &p2t_map, particles_filtered, event);
        eff = Some(new_eff);
    }
    eff
}

/// Parses the "montecarlo" field from the event dict.
/// Expects that "montecarlo" contains a "description" key (a list of field names)
/// and a "particles" key (a list of lists of values).
/// For each particle record, uses the description to build a mapping and then
/// creates an MCParticle with the correct flags. Also builds a mapping from each MCParticle to its associated hits.
pub fn parse_montecarlo(py: Python, dict: &PyDict, hits: &[Hit])
    -> PyResult<(Vec<MCParticle>, HashMap<MCParticle, Vec<Hit>>)>
{
    let mc_obj = dict.get_item("montecarlo")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("montecarlo key missing"))?
        .downcast::<PyDict>()?;
    
    // Extract the description list (field names).
    let description: Vec<String> = mc_obj.get_item("description")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("description key missing in montecarlo"))?
        .extract()?;
    
    // Extract the particles data: each particle is represented as a list.
    let particles_data: Vec<Vec<PyObject>> = mc_obj.get_item("particles")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("particles key missing in montecarlo"))?
        .extract()?;
    
    let mut particles = Vec::new();
    let mut mcp_to_hits = HashMap::new();
    
    // Iterate over each particle record.
    for p in particles_data.into_iter() {
        let mut d: HashMap<String, PyObject> = HashMap::new();
        // Build a mapping from field name to value.
        for (i, key) in description.iter().enumerate() {
            let value = p.get(i)
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyIndexError, _>("Particle data missing element"))?
                .to_object(py);
            d.insert(key.clone(), value);
        }
        
        // Extract numeric fields.
        let pkey: u64 = d.get("key").and_then(|obj| obj.extract(py).ok()).unwrap_or(0);
        let pid: i32 = d.get("pid").and_then(|obj| obj.extract(py).ok()).unwrap_or(0);
        let p_val: f64 = d.get("p").and_then(|obj| obj.extract(py).ok()).unwrap_or(0.0);
        let pt: f64 = d.get("pt").and_then(|obj| obj.extract(py).ok()).unwrap_or(0.0);
        let eta: f64 = d.get("eta").and_then(|obj| obj.extract(py).ok()).unwrap_or(0.0);
        let phi: f64 = d.get("phi").and_then(|obj| obj.extract(py).ok()).unwrap_or(0.0);
        let charge: i32 = d.get("charge").and_then(|obj| obj.extract(py).ok()).unwrap_or(0);
        
        // Extract the hit indices for this particle.
        let hit_indices: Vec<usize> = d.get("hits")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(vec![]);
        let trackhits: Vec<Hit> = hit_indices
            .iter()
            .filter_map(|&i| hits.get(i).cloned())
            .collect();
        
        // Create a new MCParticle (this sets default flags).
        let mut mcp = MCParticle::new(pkey, pid, p_val, pt, eta, phi, charge, trackhits.clone());
        
        // Set boolean flags by extracting integer values and converting them.
        // (Treat nonzero as true.)
        mcp.islong = d.get("isLong")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.isdown = d.get("isDown")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.isvelo = d.get("hasVelo")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(true); // default true as in python
        mcp.isut = d.get("hasUT")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.has_scifi = d.get("hasScifi")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.fromb = d.get("fromBeautyDecay")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.fromcharm = d.get("fromCharmDecay")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.strange = d.get("fromStrangeDecay")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        // over5 is computed from p_val: true if absolute p > 5000.
        mcp.over5 = p_val.abs() > 5000.0;
        
        // Insert the MCParticle and its associated hits.
        mcp_to_hits.insert(mcp.clone(), trackhits);
        particles.push(mcp);
    }
    Ok((particles, mcp_to_hits))
}