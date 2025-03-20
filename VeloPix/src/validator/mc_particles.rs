use pyo3::prelude::*;
use ordered_float::OrderedFloat;
use std::hash::{Hash, Hasher};
use crate::event_model::hit::Hit; 

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
