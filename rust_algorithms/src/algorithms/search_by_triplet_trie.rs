use pyo3::prelude::*;
use crate::event_model::event::Event;
use crate::event_model::hit::Hit;
use crate::event_model::module::Module;
use crate::event_model::track::Track;

#[pyclass]
struct SearchByTripletTrie {
    
}