use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use pyo3::types::PyDict;
use pyo3::prelude::*;

use crate::event_model::track::Track;
use crate::event_model::hit::Hit;
use crate::validator::efficientcy::Efficiency;
use crate::validator::mc_particles::MCParticle;
use crate::validator::event::ValidatorEvent;

pub fn comp_weights(
    tracks: &[Track],
    event: &ValidatorEvent,
) -> Result<Vec<Vec<f64>>, String> {
    let nparticles = event.particles.as_ref().map(|v| v.len()).unwrap_or(0);
    let mut weights = vec![vec![0.0; nparticles]; tracks.len()];
    if nparticles == 0 {
        return Ok(weights);
    }
    let hit_to_mcp = event.hit_to_mcp.as_ref().ok_or("Error: hit_to_mcp mapping is missing")?;

    // Build a hit id 
    let mut hit_map: HashMap<i32, &Vec<MCParticle>> = HashMap::new();
    for (hit, mcps) in hit_to_mcp.iter() {
        hit_map.insert(hit.id, mcps);
    }

    let particles = event.particles.as_ref().unwrap();
    let particle_index_map: HashMap<&MCParticle, usize> =
        particles.iter().enumerate().map(|(i, p)| (p, i)).collect();

    // For each track -> count for each particle by iterating over its hits
    for (i, track) in tracks.iter().enumerate() {
        let nhits = track.hits.len();
        if nhits < 2 {
            continue;
        }
        let mut counts = vec![0; nparticles];
        for hit in &track.hits {
            // Look up mcps
            if let Some(mcps) = hit_map.get(&hit.id) {
                // For each hit, count each particle only once!!
                let mut seen_indices = Vec::with_capacity(mcps.len());
                for mcp in *mcps {
                    if let Some(&j) = particle_index_map.get(mcp) {
                        if !seen_indices.contains(&j) {
                            seen_indices.push(j);
                        }
                    }
                }
                for j in seen_indices {
                    counts[j] += 1;
                }
            } else {
                return Err(format!(
                    "Error: hit with id {} not found in hit_to_mcp mapping",
                    hit.id
                ));
            }
        }
        for j in 0..nparticles {
            weights[i][j] = counts[j] as f64 / nhits as f64;
        }
    }
    Ok(weights)
}

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

    // For each track find the particle with maximum weight
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

pub fn reconstructed(
    p2t: &HashMap<MCParticle, (f64, Option<Track>)>,
) -> Vec<Track> {
    p2t.values()
        .filter_map(|&(_w, ref t_opt)| t_opt.clone())
        .collect()
}

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
    particle_to_tracks
        .into_iter()
        .filter(|(_, tracks)| tracks.len() > 1)
        .collect()
}

pub fn hit_efficiency(
    t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
    event: &ValidatorEvent,
) -> HashMap<Track, f64> {
    let mut hit_eff: HashMap<Track, f64> = HashMap::new();

    if let (Some(ref _hit_to_mcp), Some(ref mcp_to_hits)) =
        (&event.hit_to_mcp, &event.mcp_to_hits)
    {
        let particle_hit_ids: HashMap<MCParticle, HashSet<i32>> = mcp_to_hits
            .iter()
            .map(|(particle, hits)| {
                (
                    particle.clone(),
                    hits.iter().map(|hit| hit.id).collect::<HashSet<_>>(),
                )
            })
            .collect();

        for (track, &(_w, ref opt_particle)) in t2p.iter() {
            if let Some(ref particle) = opt_particle {
                if let Some(hit_ids) = particle_hit_ids.get(particle) {
                    let hits_from_particle = track.hits.iter().filter(|hit| hit_ids.contains(&hit.id)).count();
                    let total_hits = hit_ids.len();
                    if total_hits > 0 {
                        hit_eff.insert(track.clone(), hits_from_particle as f64 / total_hits as f64);
                    }
                }
            }
        }
    }
    hit_eff
}

pub fn update_efficiencies(
    mut eff: Option<Efficiency>,
    event: &ValidatorEvent,
    tracks: &[Track],
    weights: &[Vec<f64>],
    label: &str,
    cond: for<'a> fn(&'a MCParticle) -> bool,
    verbose: bool
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
        if verbose {
            println!("update_efficiencies: No particles match condition for '{}'", label);
        }
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
/// Optimized by precomputing a mapping from field names (from "description") to indices,
/// and then directly extracting values from each particle record.
pub fn parse_montecarlo(py: Python, dict: &PyDict, hits: &[Hit])
    -> PyResult<(Vec<MCParticle>, HashMap<MCParticle, Vec<Hit>>)>
{
    let mc_obj = dict.get_item("montecarlo")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("montecarlo key missing"))?
        .downcast::<PyDict>()?;

    let description: Vec<String> = mc_obj.get_item("description")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("description key missing in montecarlo"))?
        .extract()?;

    let particles_data: Vec<Vec<PyObject>> = mc_obj.get_item("particles")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("particles key missing in montecarlo"))?
        .extract()?;

    let mut field_index = HashMap::new();
    for (i, field) in description.iter().enumerate() {
        field_index.insert(field.as_str(), i);
    }

    let mut particles = Vec::with_capacity(particles_data.len());
    let mut mcp_to_hits = HashMap::with_capacity(particles_data.len());

    for p in particles_data.into_iter() {
        let get_field = |name: &str| -> Option<&PyObject> {
            field_index.get(name).and_then(|&i| p.get(i))
        };

        let pkey: u64 = get_field("key")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0);
        let pid: i32 = get_field("pid")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0);
        let p_val: f64 = get_field("p")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0.0);
        let pt: f64 = get_field("pt")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0.0);
        let eta: f64 = get_field("eta")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0.0);
        let phi: f64 = get_field("phi")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0.0);
        let charge: i32 = get_field("charge")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or(0);

        let hit_indices: Vec<usize> = get_field("hits")
            .and_then(|obj| obj.extract(py).ok())
            .unwrap_or_default();
        let trackhits: Vec<Hit> = hit_indices
            .iter()
            .filter_map(|&i| hits.get(i).cloned())
            .collect();

        let mut mcp = MCParticle::new(pkey, pid, p_val, pt, eta, phi, charge, trackhits.clone());

        mcp.islong = get_field("isLong")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.isdown = get_field("isDown")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.isvelo = get_field("hasVelo")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(true);
        mcp.isut = get_field("hasUT")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.has_scifi = get_field("hasScifi")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.fromb = get_field("fromBeautyDecay")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.fromcharm = get_field("fromCharmDecay")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.strange = get_field("fromStrangeDecay")
            .and_then(|obj| obj.extract::<i32>(py).ok())
            .map(|v| v != 0)
            .unwrap_or(false);
        mcp.over5 = p_val.abs() > 5000.0;

        mcp_to_hits.insert(mcp.clone(), trackhits);
        particles.push(mcp);
    }
    Ok((particles, mcp_to_hits))
}
