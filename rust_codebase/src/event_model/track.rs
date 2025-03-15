use pyo3::prelude::*;
use crate::event_model::hit::Hit;
use std::hash::{Hash, Hasher};

#[pyclass]
#[derive(Clone)]
pub struct Track {
    #[pyo3(get)]
    pub hits: Vec<Hit>,
    #[pyo3(get)]
    pub missed_last_module: bool,
    #[pyo3(get)]
    pub missed_penultimate_module: bool,
    pub missed_modules: u8
}

#[pymethods]
impl Track {
    #[new]
    pub fn new(hits: Vec<Hit>) -> Self {
        let missed_modules = 0;
        Track {
            hits, 
            missed_last_module: false, // these have been set to false in the original python implementation consider why and wheter this is still valid
            missed_penultimate_module: false,
            missed_modules
        }
    }

    pub fn add_hit(&mut self, hit: Hit) {
        self.hits.push(hit);
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Track with {} hits, missed_modules: {}",
            self.hits.len(),
            self.missed_modules
        ))
    }
}

// needed for validator
impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.hits == other.hits &&
        self.missed_last_module == other.missed_last_module &&
        self.missed_penultimate_module == other.missed_penultimate_module
    }
}

// needed for validator
impl Eq for Track {}

// needed for validator
impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hits.hash(state);
        self.missed_last_module.hash(state);
        self.missed_penultimate_module.hash(state);
    }
}