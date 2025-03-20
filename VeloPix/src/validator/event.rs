use pyo3::prelude::*;
use std::collections::HashMap;
use crate::event_model::hit::Hit;
use crate::validator::mc_particles::MCParticle;

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
    pub mcp_to_hits: Option<HashMap<MCParticle, Vec<Hit>>>,
    pub hit_to_mcp: Option<HashMap<Hit, Vec<MCParticle>>>,
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
        particles: Option<Vec<MCParticle>>,
    ) -> Self {
        let mut hit_to_mcp = None;
        let computed_particles = if let Some(ref mcp_hits) = mcp_to_hits {
            let p: Vec<MCParticle> = mcp_hits.keys().cloned().collect();
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
            p
        } else {
            Vec::new()
        };

        // If particles were provided explicitly, use them; otherwise, use computed_particles.
        let particles = if let Some(p) = particles {
            p
        } else {
            computed_particles
        };

        ValidatorEvent {
            module_prefix_sum,
            hit_xs,
            hit_ys,
            hit_zs,
            hits,
            mcp_to_hits,
            hit_to_mcp,
            particles: Some(particles),
        }
    }

    pub fn get_hit(&self, hit_id: usize) -> Option<Hit> {
        self.hits.get(hit_id).cloned()
    }
}

