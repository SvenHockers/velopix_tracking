use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::PyDict;
use itertools::iproduct;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use ordered_float::OrderedFloat;
use std::fmt;

// Import your existing Hit and Track types.
use crate::event_model::hit::Hit;
use crate::event_model::track::Track;

/// A SOA datastructure for events (similar to the original Python validator_event).
#[pyclass]
#[derive(Clone, Debug)]
pub struct ValidatorEvent {
    #[pyo3(get)]
    pub module_prefix_sum: Vec<usize>,
    #[pyo3(get)]
    pub hit_xs: Vec<f64>,
    #[pyo3(get)]
    pub hit_ys: Vec<f64>,
    #[pyo3(get)]
    pub hit_zs: Vec<f64>,
    #[pyo3(get)]
    pub hits: Vec<Hit>,
    /// Mapping from MCParticle to its hits.
    pub mcp_to_hits: Option<HashMap<MCParticle, Vec<Hit>>>,
    /// Mapping from Hit to associated MCParticles.
    pub hit_to_mcp: Option<HashMap<Hit, Vec<MCParticle>>>,
    /// The list of MCParticles (extracted from mcp_to_hits keys).
    pub particles: Option<Vec<MCParticle>>,
}

#[pymethods]
impl ValidatorEvent {
    #[new]
    pub fn new(
        module_prefix_sum: Vec<usize>,
        hit_xs: Vec<f64>,
        hit_ys: Vec<f64>,
        hit_zs: Vec<f64>,
        hits: Vec<Hit>,
        mcp_to_hits: Option<HashMap<MCParticle, Vec<Hit>>>,
    ) -> Self {
        let mut hit_to_mcp = None;
        let mut particles = None;
        if let Some(ref mcp_hits) = mcp_to_hits {
            particles = Some(mcp_hits.keys().cloned().collect());
            let mut ht = HashMap::new();
            for h in hits.iter() {
                ht.insert(h.clone(), Vec::new());
            }
            for (mcp, mhits) in mcp_hits.iter() {
                for h in mhits.iter() {
                    if let Some(vec) = ht.get_mut(h) {
                        vec.push(mcp.clone());
                    }
                }
            }
            hit_to_mcp = Some(ht);
        }
        ValidatorEvent {
            module_prefix_sum,
            hit_xs,
            hit_ys,
            hit_zs,
            hits,
            mcp_to_hits,
            hit_to_mcp,
            particles,
        }
    }

    /// Retrieve a hit by its index.
    pub fn get_hit(&self, hit_id: usize) -> Option<Hit> {
        self.hits.get(hit_id).cloned()
    }
}

/// MCParticle used in validation.
/// f64 fields are wrapped in OrderedFloat to allow deriving Eq/Hash;
/// custom getters return plain f64 values.
#[pyclass]
#[derive(Clone, Debug)]
pub struct MCParticle {
    #[pyo3(get)]
    pub pkey: u64,
    #[pyo3(get)]
    pub pid: i32,
    pub p: OrderedFloat<f64>,
    pub pt: OrderedFloat<f64>,
    pub eta: OrderedFloat<f64>,
    pub phi: OrderedFloat<f64>,
    #[pyo3(get)]
    pub charge: i32,
    pub velohits: Vec<Hit>,
    #[pyo3(get)]
    pub islong: bool,
    #[pyo3(get)]
    pub isdown: bool,
    #[pyo3(get)]
    pub isvelo: bool,
    #[pyo3(get)]
    pub isut: bool,
    #[pyo3(get)]
    pub has_scifi: bool,
    #[pyo3(get)]
    pub strange: bool,
    #[pyo3(get)]
    pub fromb: bool,
    #[pyo3(get)]
    pub fromcharm: bool,
    #[pyo3(get)]
    pub over5: bool,
}

impl PartialEq for MCParticle {
    fn eq(&self, other: &Self) -> bool {
        self.pkey == other.pkey
    }
}

impl Eq for MCParticle {}

impl Hash for MCParticle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pkey.hash(state);
    }
}

#[pymethods]
impl MCParticle {
    #[new]
    pub fn new(
        pkey: u64,
        pid: i32,
        p: f64,
        pt: f64,
        eta: f64,
        phi: f64,
        charge: i32,
        velohits: Vec<Hit>,
    ) -> Self {
        let over5 = p.abs() > 5000.0;
        MCParticle {
            pkey,
            pid,
            p: OrderedFloat::from(p),
            pt: OrderedFloat::from(pt),
            eta: OrderedFloat::from(eta),
            phi: OrderedFloat::from(phi),
            charge,
            velohits,
            islong: false,
            isdown: false,
            isvelo: false,
            isut: false,
            has_scifi: false,
            strange: false,
            fromb: false,
            fromcharm: false,
            over5,
        }
    }

    #[getter]
    fn get_p(&self) -> PyResult<f64> {
        Ok(self.p.into_inner())
    }

    #[getter]
    fn get_pt(&self) -> PyResult<f64> {
        Ok(self.pt.into_inner())
    }

    #[getter]
    fn get_eta(&self) -> PyResult<f64> {
        Ok(self.eta.into_inner())
    }

    #[getter]
    fn get_phi(&self) -> PyResult<f64> {
        Ok(self.phi.into_inner())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "MCParticle {}:\n\tpid: {}\n\tp: {}\n\tpt: {}\n\teta: {}\n\tphi: {}",
            self.pkey,
            self.pid,
            self.p.into_inner(),
            self.pt.into_inner(),
            self.eta.into_inner(),
            self.phi.into_inner()
        ))
    }
}

/// Efficiency structure for validation.
#[pyclass]
#[derive(Debug)]
pub struct Efficiency {
    #[pyo3(get)]
    pub label: String,
    #[pyo3(get)]
    pub n_particles: usize,
    #[pyo3(get)]
    pub n_reco: usize,
    #[pyo3(get)]
    pub n_pure: usize,
    #[pyo3(get)]
    pub n_clones: usize,
    #[pyo3(get)]
    pub n_events: usize,
    #[pyo3(get)]
    pub n_heff: usize,
    #[pyo3(get)]
    pub n_hits: usize,
    #[pyo3(get)]
    pub recoeffT: f64,
    #[pyo3(get)]
    pub purityT: f64,
    #[pyo3(get)]
    pub avg_recoeff: f64,
    #[pyo3(get)]
    pub avg_purity: f64,
    #[pyo3(get)]
    pub avg_hiteff: f64,
}

#[pymethods]
impl Efficiency {
    #[new]
    pub fn new(
        t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
        p2t: &HashMap<MCParticle, (f64, Option<Track>)>,
        particles: Vec<MCParticle>,
        event: &ValidatorEvent,
        label: String,
    ) -> Self {
        let mut eff = Efficiency {
            label,
            n_particles: 0,
            n_reco: 0,
            n_pure: 0,
            n_clones: 0,
            n_events: 0,
            n_heff: 0,
            n_hits: 0,
            recoeffT: 0.0,
            purityT: 0.0,
            avg_recoeff: 0.0,
            avg_purity: 0.0,
            avg_hiteff: 0.0,
        };
        eff.add_event(t2p, p2t, particles, event);
        eff
    }

    pub fn add_event(&mut self,
                     t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
                     p2t: &HashMap<MCParticle, (f64, Option<Track>)>,
                     particles: Vec<MCParticle>,
                     event: &ValidatorEvent) {
        self.n_events += 1;
        self.n_particles += particles.len();
        self.n_reco += reconstructed(p2t).len();
        self.avg_recoeff = 100.0 * (self.n_reco as f64 / self.n_particles as f64);
        self.n_clones += clones(t2p.clone()).values().map(|v| v.len().saturating_sub(1)).sum::<usize>();
        let hit_eff = hit_efficiency(t2p, event);
        let purities: Vec<f64> = t2p.values().filter_map(|&(w, ref opt)| {
            if opt.is_some() { Some(w) } else { None }
        }).collect();
        self.n_pure += purities.iter().sum::<f64>() as usize;
        self.n_heff += hit_eff.values().sum::<f64>() as usize;
        self.n_hits += hit_eff.len();
        if !hit_eff.is_empty() {
            self.avg_hiteff = 100.0 * (hit_eff.values().sum::<f64>() / hit_eff.len() as f64);
        }
        if !purities.is_empty() {
            self.avg_purity = 100.0 * (purities.iter().sum::<f64>() / purities.len() as f64);
        }
        if self.n_particles > 0 {
            self.recoeffT = 100.0 * self.n_reco as f64 / self.n_particles as f64;
        }
        if self.n_reco > 0 {
            self.purityT = 100.0 * self.n_pure as f64 / ((self.n_reco as f64) + (self.n_clones as f64));
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }
}

impl fmt::Display for Efficiency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clone_percentage = if self.n_reco > 0 {
            100.0 * self.n_clones as f64 / self.n_reco as f64
        } else {
            0.0
        };
        let hit_eff = if self.n_hits > 0 {
            100.0 * self.n_heff as f64 / self.n_hits as f64
        } else {
            0.0
        };
        write!(f,
            "{:18} : {} from {} ({:.1}%, {:.1}%) {} clones ({:.2}%), purity: ({:.2}%, {:.2}%),  hitEff: ({:.2}%, {:.2}%)",
            self.label,
            self.n_reco,
            self.n_particles,
            self.recoeffT,
            self.avg_recoeff,
            self.n_clones,
            clone_percentage,
            self.purityT,
            self.avg_purity,
            self.avg_hiteff,
            hit_eff
        )
    }
}

// -----------------------------------------------------------------
// Helper functions (adapted from Python logic)
// -----------------------------------------------------------------

/// Compute w(t,p): the fraction of hits on track t contributed by particle p.
fn comp_weights(tracks: &[Track], event: &ValidatorEvent) -> Vec<Vec<f64>> {
    let particles = event.particles.as_ref().unwrap_or(&vec![]);
    let n_particles = particles.len();
    let mut w = vec![vec![0.0; n_particles]; tracks.len()];
    for (i, track) in tracks.iter().enumerate() {
        let nhits = track.hits.len();
        if nhits < 2 { continue; }
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
/// Returns two hash maps: one mapping each Track to (max_weight, Option<MCParticle>)
/// and one mapping each MCParticle to (max_weight, Option<Track>).
fn hit_purity(
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
            .fold((0.0, 0), |(mw, mi), (idx, &w)| if w > mw { (w, idx) } else { (mw, mi) });
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
            .fold((0.0, 0), |(mw, mi), (idx, &w)| if w > mw { (w, idx) } else { (mw, mi) });
        if max_w > 0.7 && max_idx < tracks.len() {
            p2t.insert(particle.clone(), (max_w, Some(tracks[max_idx].clone())));
        } else {
            p2t.insert(particle.clone(), (max_w, None));
        }
    }
    (t2p, p2t)
}

/// Compute hit efficiency for each (track, particle) pair.
fn hit_efficiency(
    t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
    event: &ValidatorEvent,
) -> HashMap<(Track, MCParticle), f64>
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut hit_eff = HashMap::new();
    for (track, &(_, ref opt_particle)) in t2p.iter() {
        if let Some(ref particle) = opt_particle {
            let hits_p_on_t = track.hits.iter().filter(|h| {
                event.hit_to_mcp
                    .as_ref()
                    .and_then(|m| m.get(h))
                    .map(|vec| vec.contains(particle))
                    .unwrap_or(false)
            }).count();
            if let Some(m_hits) = event.mcp_to_hits.as_ref().and_then(|m| m.get(particle)) {
                hit_eff.insert((track.clone(), particle.clone()),
                    hits_p_on_t as f64 / m_hits.len() as f64);
            }
        }
    }
    hit_eff
}

/// Return all reconstructed tracks (tracks with an associated particle).
fn reconstructed(
    p2t: &HashMap<MCParticle, (f64, Option<Track>)>,
) -> Vec<Track>
where
    Track: Clone,
    MCParticle: Clone,
{
    p2t.values().filter_map(|&(_, ref opt)| opt.clone()).collect()
}

/// Returns a mapping of particles that have clones (more than one associated track).
fn clones(
    t2p: HashMap<Track, (f64, Option<MCParticle>)>,
) -> HashMap<MCParticle, Vec<Track>>
where
    Track: Clone + Hash + Eq,
    MCParticle: Clone + Hash + Eq,
{
    let mut p2t_map = HashMap::new();
    for (track, &(_, ref opt_particle)) in t2p.into_iter() {
        if let Some(particle) = opt_particle {
            p2t_map.entry(particle.clone()).or_insert_with(Vec::new).push(track.clone());
        }
    }
    p2t_map.into_iter().filter(|(_, v)| v.len() > 1).collect()
}

/// Returns ghost tracks (tracks with no associated particle).
fn ghosts(
    t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
) -> Vec<Track>
where
    Track: Clone,
{
    t2p.iter()
        .filter_map(|(track, &(_, ref opt_particle))| if opt_particle.is_none() { Some(track.clone()) } else { None })
        .collect()
}

/// Returns the ghost rate (fraction of ghost tracks and their count).
fn ghost_rate(
    t2p: HashMap<Track, (f64, Option<MCParticle>)>,
) -> (f64, usize) {
    let ntracks = t2p.len();
    let ghost_tracks = ghosts(&t2p);
    let nghosts = ghost_tracks.len();
    (nghosts as f64 / ntracks as f64, nghosts)
}

/// Update and accumulate efficiency information for a given event.
/// The condition is provided as a function pointer: fn(&MCParticle) -> bool.
fn update_efficiencies(
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
    let particles: Vec<MCParticle> = event.particles
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter(|p| cond(p))
        .cloned()
        .collect();
    if particles.is_empty() {
        return eff;
    }
    let pidx_filtered: Vec<usize> = event.particles
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
        .filter_map(|(i, p)| if cond(p) { Some(i) } else { None })
        .collect();
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

// -----------------------------------------------------------------
// Exposed Python functions
// -----------------------------------------------------------------

/// Exposed to Python: prints validation information (ghost rate and efficiency summaries)
/// for a set of events (provided as JSON dictionaries) and corresponding track lists.
#[pyfunction]
pub fn validate_print(py_events: Vec<&PyDict>, py_tracks: Vec<Vec<Track>>) -> PyResult<()> {
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
        let event = ValidatorEvent::new(module_prefix_sum, hit_xs, hit_ys, hit_zs, hits, None);
        tracking_data.push(event);
    }
    let mut n_tracks = 0;
    let mut avg_ghost_rate = 0.0;
    let mut n_allghosts = 0;
    let mut eff_velo: Option<Efficiency> = None;
    let mut eff_long: Option<Efficiency> = None;
    let mut eff_long5: Option<Efficiency> = None;
    let mut eff_long_strange: Option<Efficiency> = None;
    let mut eff_long_strange5: Option<Efficiency> = None;
    let mut eff_long_fromb: Option<Efficiency> = None;
    let mut eff_long_fromb5: Option<Efficiency> = None;
    for (event, tracks) in tracking_data.iter().zip(py_tracks.iter()) {
        n_tracks += tracks.len();
        let weights = comp_weights(tracks, event);
        let (t2p_map, _) = hit_purity(tracks, event.particles.as_ref().unwrap_or(&vec![]), &weights);
        // Pass ownership (by cloning) to ghost_rate.
        let (grate, nghosts) = ghost_rate(t2p_map.clone());
        n_allghosts += nghosts;
        avg_ghost_rate += grate;
        // Explicitly cast closures to function pointers.
        eff_velo = update_efficiencies(eff_velo, event, tracks, &weights, "velo",
            (|p: &MCParticle| p.isvelo && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
        eff_long = update_efficiencies(eff_long, event, tracks, &weights, "long",
            (|p: &MCParticle| p.islong && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
        eff_long5 = update_efficiencies(eff_long5, event, tracks, &weights, "long>5GeV",
            (|p: &MCParticle| p.islong && p.over5 && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
        eff_long_strange = update_efficiencies(eff_long_strange, event, tracks, &weights, "long_strange",
            (|p: &MCParticle| p.islong && p.strange && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
        eff_long_strange5 = update_efficiencies(eff_long_strange5, event, tracks, &weights, "long_strange>5GeV",
            (|p: &MCParticle| p.islong && p.over5 && p.strange && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
        eff_long_fromb = update_efficiencies(eff_long_fromb, event, tracks, &weights, "long_fromb",
            (|p: &MCParticle| p.islong && p.fromb && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
        eff_long_fromb5 = update_efficiencies(eff_long_fromb5, event, tracks, &weights, "long_fromb>5GeV",
            (|p: &MCParticle| p.islong && p.over5 && p.fromb && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool);
    }
    let nevents = tracking_data.len();
    println!(
        "{} tracks including {} ghosts ({:.1}%). Event average {:.1}%",
        n_tracks,
        n_allghosts,
        100.0 * n_allghosts as f64 / n_tracks as f64,
        100.0 * avg_ghost_rate / nevents as f64
    );
    if let Some(e) = eff_velo { println!("{}", e); }
    if let Some(e) = eff_long { println!("{}", e); }
    if let Some(e) = eff_long5 { println!("{}", e); }
    if let Some(e) = eff_long_strange { println!("{}", e); }
    if let Some(e) = eff_long_strange5 { println!("{}", e); }
    if let Some(e) = eff_long_fromb { println!("{}", e); }
    if let Some(e) = eff_long_fromb5 { println!("{}", e); }
    Ok(())
}

/// Exposed to Python: computes and returns the reconstruction efficiency (value in [0,1])
/// for the given particle type from a set of JSON event dictionaries and corresponding tracks.
#[pyfunction]
pub fn validate_efficiency(py_events: Vec<&PyDict>, py_tracks: Vec<Vec<Track>>, particle_type: &str) -> PyResult<f64> {
    let mut eff: Option<Efficiency> = None;
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
        let event = ValidatorEvent::new(module_prefix_sum, hit_xs, hit_ys, hit_zs, hits, None);
        tracking_data.push(event);
    }
    for (event, tracks) in tracking_data.iter().zip(py_tracks.iter()) {
        let weights = comp_weights(tracks, event);
        let cond = match particle_type {
            "velo" => (|p: &MCParticle| p.isvelo && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            "long" => (|p: &MCParticle| p.islong && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            "long>5GeV" => (|p: &MCParticle| p.islong && p.over5 && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            "long_strange" => (|p: &MCParticle| p.islong && p.strange && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            "long_strange>5GeV" => (|p: &MCParticle| p.islong && p.over5 && p.strange && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            "long_fromb" => (|p: &MCParticle| p.islong && p.fromb && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            "long_fromb>5GeV" => (|p: &MCParticle| p.islong && p.over5 && p.fromb && (p.pid.abs() != 11)) as fn(&MCParticle) -> bool,
            _ => |_| false,
        };
        eff = update_efficiencies(eff, event, tracks, &weights, particle_type, cond);
    }
    if let Some(eff_obj) = eff {
        Ok(eff_obj.recoeffT / 100.0)
    } else {
        Ok(0.0)
    }
}
