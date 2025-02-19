use std::fmt::format;

use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Hit {
    #[pyo3(get)]
    pub x: f64,
    #[pyo3(get)]
    pub y: f64,
    #[pyo3(get)]
    pub z: f64,
    #[pyo3(get)]
    pub t: f64,
    #[pyo3(get)]
    pub id: u32,
    #[pyo3(get)]
    pub module_number: i32,
    #[pyo3(get)]
    pub with_t: bool
}

#[pymethods]
impl Hit {
    #[new]
    pub fn new(
        x: f64, 
        y: f64, 
        z: f64, 
        hit_id: u32, 
        module: Option<i32>,
        t: Option<f64>, 
        with_t: Option<bool>
    ) -> Self {
        Hit {
            x, y, z, t: t.unwrap_or(0.0), id: hit_id, module_number: module.unwrap_or(-1), with_t: with_t.unwrap_or(false)
        }
    }

    pub fn __repr__(&self) -> PyResult<String> {
        let rep = if self.with_t {
            format!("#{} module {} {{{}, {}, {}, {}}}", self.id, self.module_number, self.x, self.y, self.z, self.t)
        } else {
            format!("#{} module {} {{{}, {}, {}}}", self.id, self.module_number, self.x, self.y, self.z)
        };
        Ok(rep)
    }
}