// helpers.rs
use itertools::iproduct;
use std::collections::HashMap;
use std::hash::Hash;

// Import the types defined in your validator module.
// Adjust the module path if needed.
use crate::validator::{ValidatorEvent, MCParticle, Efficiency};
use crate::event_model::track::Track;

/// Compute the weight matrix for tracks versus particles.
/// Each entry w(t,p) is the fraction of hits on track t contributed by particle p.
pub fn comp_weights(tracks: &[Track], event: &ValidatorEvent) -> Vec<Vec<f64>> {
    let particles = match &event.particles {
        Some(ps) => ps,
        None => &vec![],
    };
    let n_particles = particles.len();
    let mut w = vec![vec![0.0; n_particles]; tracks.len()];
    for (i, track) in tracks.iter().enumerate() {
        let nhits = track.hits.len();
        if nhits < 2 {
            continue;
        }
        for j in 0..n_particles {
            let particle = &particles[j];
            let nhits_from_p = track.hits.iter().filter(|h| {
                event.hit_to_mcp
                    .as_ref()
                    .and_then(|m| m.get(h))
                    .map(|vec| vec.contains(particle))
                    .unwrap_or(false)
            }).count();
            w[i][j] = nhits_from_p as f64 / nhits as f64;
        }
    }
    w
}

/// Calculate hit purity tables.
/// Returns two hash maps:
///  - One mapping each Track to (max_weight, Option<MCParticle>).
///  - One mapping each MCParticle to (max_weight, Option<Track>).
pub fn hit_purity(
    tracks: &[Track],
    particles: &[MCParticle],
    weights: &[Vec<f64>],
) -> (HashMap<Track, (f64, Option<MCParticle>)>, HashMap<MCParticle, (f64, Option<Track>)>)
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut t2p = HashMap::new();
    let mut p2t = HashMap::new();
    for track in tracks.iter() {
        t2p.insert(track.clone(), (0.0, None));
    }
    for particle in particles.iter() {
        p2t.insert(particle.clone(), (0.0, None));
    }
    for (i, track) in tracks.iter().enumerate() {
        let (max_w, max_idx) = weights[i]
            .iter()
            .enumerate()
            .fold((0.0, 0), |(mw, mi), (idx, &w)| {
                if w > mw { (w, idx) } else { (mw, mi) }
            });
        if max_w > 0.7 && max_idx < particles.len() {
            t2p.insert(track.clone(), (max_w, Some(particles[max_idx].clone())));
        } else {
            t2p.insert(track.clone(), (max_w, None));
        }
    }
    for (j, particle) in particles.iter().enumerate() {
        let col: Vec<f64> = weights.iter().map(|row| row[j]).collect();
        let (max_w, max_idx) = col
            .iter()
            .enumerate()
            .fold((0.0, 0), |(mw, mi), (idx, &w)| {
                if w > mw { (w, idx) } else { (mw, mi) }
            });
        if max_w > 0.7 && max_idx < tracks.len() {
            p2t.insert(particle.clone(), (max_w, Some(tracks[max_idx].clone())));
        } else {
            p2t.insert(particle.clone(), (max_w, None));
        }
    }
    (t2p, p2t)
}

/// Compute hit efficiency for each (track, particle) pair.
/// Returns a HashMap mapping (Track, MCParticle) pairs to efficiency.
pub fn hit_efficiency(
    t2p: HashMap<Track, (f64, Option<MCParticle>)>,
    event: &ValidatorEvent,
) -> HashMap<(Track, MCParticle), f64>
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut hit_eff = HashMap::new();
    for (track, (w, opt_particle)) in t2p.into_iter() {
        if let Some(particle) = opt_particle {
            let hits_p_on_t = track.hits.iter().filter(|h| {
                event.hit_to_mcp
                    .as_ref()
                    .and_then(|m| m.get(h))
                    .map(|vec| vec.contains(&particle))
                    .unwrap_or(false)
            }).count();
            if let Some(m_hits) = event.mcp_to_hits.as_ref().and_then(|m| m.get(&particle)) {
                hit_eff.insert((track.clone(), particle.clone()), hits_p_on_t as f64 / m_hits.len() as f64);
            }
        }
    }
    hit_eff
}

/// Returns ghost tracks (tracks with no associated particle).
pub fn ghosts(
    t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
) -> Vec<Track>
where
    Track: Clone,
{
    t2p.iter()
        .filter_map(|(track, &(_, ref opt_particle))| {
            if opt_particle.is_none() {
                Some(track.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Returns the ghost rate (fraction of ghost tracks and their count).
pub fn ghost_rate(
    t2p: HashMap<Track, (f64, Option<MCParticle>)>,
) -> (f64, usize)
where
    Track: Clone,
    MCParticle: Clone,
{
    let ntracks = t2p.len();
    let ghost_tracks = ghosts(&t2p);
    let nghosts = ghost_tracks.len();
    (nghosts as f64 / ntracks as f64, nghosts)
}

/// Returns a mapping of particles that have clones (i.e. more than one associated track).
pub fn clones(
    t2p: HashMap<Track, (f64, Option<MCParticle>)>,
) -> HashMap<MCParticle, Vec<Track>>
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut p2t_map = HashMap::new();
    for (track, (_, opt_particle)) in t2p.into_iter() {
        if let Some(particle) = opt_particle {
            p2t_map.entry(particle.clone()).or_insert_with(Vec::new).push(track.clone());
        }
    }
    p2t_map.into_iter().filter(|(_, v)| v.len() > 1).collect()
}

/// Update and accumulate efficiency information for a given event.
/// The condition is provided as a function pointer: fn(&MCParticle) -> bool.
/// Returns an updated Efficiency object.
pub fn update_efficiencies(
    mut eff: Option<Efficiency>,
    event: &ValidatorEvent,
    tracks: &[Track],
    weights: &[Vec<f64>],
    label: &str,
    cond: fn(&MCParticle) -> bool,
) -> Option<Efficiency>
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let particles: Vec<MCParticle> = match &event.particles {
        Some(ps) => ps.iter().filter(|p| cond(p)).cloned().collect(),
        None => vec![],
    };
    if particles.is_empty() {
        return eff;
    }
    let pidx_filtered: Vec<usize> = match &event.particles {
        Some(ps) => ps.iter().enumerate().filter_map(|(i, p)| if cond(p) { Some(i) } else { None }).collect(),
        None => vec![],
    };
    let weights_filtered: Vec<Vec<f64>> = weights.iter()
        .map(|row| pidx_filtered.iter().map(|&j| row[j]).collect())
        .collect();
    let (t2p_map, _p2t_map) = hit_purity(tracks, &particles, &weights_filtered);
    if let Some(ref mut e) = eff {
        e.n_events += 1;
    } else {
        eff = Some(Efficiency {
            label: label.to_string(),
            n_particles: particles.len(),
            n_reco: 0,
            n_pure: 0,
            n_clones: 0,
            n_events: 1,
            n_heff: 0,
            n_hits: 0,
            recoeffT: 0.0,
            purityT: 0.0,
            avg_recoeff: 0.0,
            avg_purity: 0.0,
            avg_hiteff: 0.0,
        });
    }
    eff
}
