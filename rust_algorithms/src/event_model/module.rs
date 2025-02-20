use pyo3::prelude::*;
use crate::event_model::hit::Hit;

#[pyclass]
#[derive(Clone)]
pub struct Module {
    #[pyo3(get)]
    pub module_number: u32,
    pub z: Vec<f64>,
    pub hit_start_index: usize,
    pub hit_end_index: usize,
    pub global_hits: Vec<Hit>,
}

#[pymethods]
impl Module {
    #[new]
    pub fn new(module_number: u32, z: Vec<f64>, hit_start_index: usize, hit_end_index: usize, global_hits: Vec<Hit>) -> PyResult<Self> {
        if hit_start_index > hit_end_index || hit_end_index > global_hits.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid hit indices: hit_end_index must be >= hit_start_index and within bounds of global_hits"
                ));
        }
        
        Ok(Module {
            module_number,
            z,
            hit_start_index,
            hit_end_index,
            global_hits,
        })
    }

    // return hits accociated with a specific module
    pub fn hits(&self) -> PyResult<Vec<Hit>> {
        Ok(self.global_hits[self.hit_start_index..self.hit_end_index].to_vec())
    }

    pub fn __repr__(&self) -> PyResult<String> {
        let hits = self.hits()?;
        Ok(
            format!("module {}:\n At z: {:?}\n Number of hits: {}\n Hits: {:?}",
            self.module_number,
            self.z,
            hits.len(),
            hits)
        )
    }
}
