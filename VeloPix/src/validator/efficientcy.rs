use pyo3::prelude::*;
use std::fmt;
use std::collections::HashMap;
use crate::event_model::track::Track;
use crate::validator::event::ValidatorEvent;
use crate::validator::mc_particles::MCParticle;
use crate::validator::helper;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Efficiency {
    #[pyo3(get)]
    pub label: String,
    #[pyo3(get)]
    pub n_particles: usize,
    #[pyo3(get)]
    pub n_reco: usize,
    #[pyo3(get)]
    pub n_pure: f64,
    #[pyo3(get)]
    pub n_clones: usize,
    #[pyo3(get)]
    pub n_events: usize,
    #[pyo3(get)]
    pub n_heff: f64,
    #[pyo3(get)]
    pub n_hits: usize,
    #[pyo3(get)]
    pub recoeff_t: f64,
    #[pyo3(get)]
    pub purity_t: f64,
    #[pyo3(get)]
    pub avg_recoeff: f64,
    #[pyo3(get)]
    pub avg_purity: f64,
    #[pyo3(get)]
    pub avg_hiteff: f64,
}

impl Efficiency {
    pub fn add_event_internal(
        &mut self,
        t2p: &HashMap<Track, (f64, Option<MCParticle>)>,
        p2t: &HashMap<MCParticle, (f64, Option<Track>)>,
        particles: Vec<MCParticle>,
        event: &ValidatorEvent,
    ) {
        self.n_events += 1;
        self.n_particles += particles.len();
        let reco_tracks = helper::reconstructed(p2t);
        self.n_reco += reco_tracks.len();

        if self.n_particles > 0 {
            self.avg_recoeff = 100.0 * (self.n_reco as f64 / self.n_particles as f64);
        } else {
            self.avg_recoeff = 0.0;
        }

        let clones_map = helper::clones(t2p.clone());
        let clones_count: usize = clones_map.values()
            .map(|v| v.len().saturating_sub(1))
            .sum();
        self.n_clones += clones_count;

        let hit_eff = helper::hit_efficiency(t2p, event);

        let purities: Vec<f64> = t2p.values().filter_map(|&(w, ref opt)| {
            if opt.is_some() { Some(w) } else { None }
        }).collect();

        self.n_pure += purities.iter().sum::<f64>();
        self.n_heff += hit_eff.values().sum::<f64>();
        self.n_hits += hit_eff.len();

        if !hit_eff.is_empty() {
            self.avg_hiteff = 100.0 * (hit_eff.values().sum::<f64>() / hit_eff.len() as f64);
        } else {
            self.avg_hiteff = 0.0;
        }

        if !purities.is_empty() {
            self.avg_purity = 100.0 * (purities.iter().sum::<f64>() / purities.len() as f64);
        } else {
            self.avg_purity = 0.0;
        }

        if self.n_particles > 0 {
            self.recoeff_t = 100.0 * self.n_reco as f64 / self.n_particles as f64;
        }

        if self.n_reco > 0 {
            self.purity_t = 100.0 * self.n_pure / ((self.n_reco as f64) + (self.n_clones as f64));
        }
    }
}

#[pymethods]
impl Efficiency {
    #[new]
    pub fn new(label: String) -> Self {
        Efficiency {
            label,
            n_particles: 0,
            n_reco: 0,
            n_pure: 0.0,      
            n_clones: 0,
            n_events: 0,
            n_heff: 0.0,  
            n_hits: 0,
            recoeff_t: 0.0,
            purity_t: 0.0,
            avg_recoeff: 0.0,
            avg_purity: 0.0,
            avg_hiteff: 0.0,
        }
    }


    pub fn update_from_py(
        &mut self,
        t2p_data: Vec<(Track, (f64, Option<MCParticle>))>,
        p2t_data: Vec<(MCParticle, (f64, Option<Track>))>,
        particles: Vec<MCParticle>,
        event: &ValidatorEvent,    
    ) {
        let t2p: HashMap<Track, (f64, Option<MCParticle>)> = t2p_data.into_iter().collect();
        let p2t: HashMap<MCParticle, (f64, Option<Track>)> = p2t_data.into_iter().collect();
        self.add_event_internal(&t2p, &p2t, particles, event);
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
        let hit_eff_percentage = if self.n_hits > 0 {
            100.0 * self.n_heff / self.n_hits as f64
        } else {
            0.0
        };
        write!(
            f,
            "{:18} : {} from {} ({:.1}%, {:.1}%) {} clones ({:.2}%), purity: ({:.2}%, {:.2}%),  hitEff: ({:.2}%, {:.2}%)",
            self.label,
            self.n_reco,
            self.n_particles,
            self.recoeff_t,
            self.avg_recoeff,
            self.n_clones,
            clone_percentage,
            self.purity_t,
            self.avg_purity,
            self.avg_hiteff,
            hit_eff_percentage
        )
    }
}
