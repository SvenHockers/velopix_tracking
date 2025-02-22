use pyo3::prelude::*;
use crate::event_model::hit::Hit;

#[pyclass]
#[derive(Clone)]
pub struct Track {
    #[pyo3(get)]
    pub hits: Vec<Hit>,
    #[pyo3(get)]
    pub missed_last_module: bool,
    #[pyo3(get)]
    pub missed_penultimate_module: bool
}

#[pymethods]
impl Track {
    #[new]
    pub fn new(hits: Vec<Hit>) -> Self {
        Track {
            hits, 
            missed_last_module: false, // these have been set to false in the original python implementation consider why and wheter this is still valid
            missed_penultimate_module: false,
        }
    }

    pub fn add_hit(&mut self, hit: Hit) {
        self.hits.push(hit);
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Track with {} hits: {:?}", self.hits.len(), self.hits))
    }
}