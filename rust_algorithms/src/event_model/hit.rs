use pyo3::prelude::*;
use std::hash::{Hash, Hasher};
use std::fmt;

#[pyclass]
#[derive(Clone, Debug)]  
pub struct Hit {
    #[pyo3(get)]
    pub id: i32,
    #[pyo3(get)]
    pub x: f64,
    #[pyo3(get)]
    pub y: f64,
    #[pyo3(get)]
    pub z: f64,
    #[pyo3(get)]
    pub t: f64,
    #[pyo3(get)]
    pub module_number: i32,
    #[pyo3(get)]
    pub with_t: bool,
}

#[pymethods]
impl Hit {
    /// Creates a new Hit instance.
    /// The `hit_id` is provided as a parameter and will be used for equality and display.
    #[new]
    pub fn new(
        x: f64, 
        y: f64, 
        z: f64, 
        hit_id: i32, 
        module: Option<i32>,
        t: Option<f64>, 
        with_t: Option<bool>
    ) -> Self {
        Hit {
            id: hit_id,
            x,
            y,
            z,
            t: t.unwrap_or(0.0),
            module_number: module.unwrap_or(-1),
            with_t: with_t.unwrap_or(false),
        }
    }

    pub fn __repr__(&self) -> PyResult<String> {
        if self.with_t {
            Ok(format!(
                "#{} module {} {{{}, {}, {}, {}}}",
                self.id, self.module_number, self.x, self.y, self.z, self.t
            ))
        } else {
            Ok(format!(
                "#{} module {} {{{}, {}, {}}}",
                self.id, self.module_number, self.x, self.y, self.z
            ))
        }
    }
}

// Impl PartialEq and Eq to compare Hit instances.
impl PartialEq for Hit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Hit {}

// Impl Hash so that Hit can be used in hash-based collections.
impl Hash for Hit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// Used in the graph_DFS algo 
impl fmt::Display for Hit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Write output here")
    }
}
