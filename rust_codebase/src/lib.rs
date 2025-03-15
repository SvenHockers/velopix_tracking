use pyo3::prelude::*;

mod algorithms;
mod event_model;
mod validator;

use crate::event_model::hit::Hit;
use crate::event_model::track::Track;
use crate::event_model::module::Module;
use crate::event_model::event::Event;
use crate::algorithms::track_following::TrackFollowing;
use crate::algorithms::graph_dfs::GraphDFS;
use crate::algorithms::search_by_triplet_trie::SearchByTripletTrie;
use crate::validator::validator::{ validate_print, validate_efficiency, validate_to_json, validate_to_json_nested };

#[pymodule]
fn velopix_tracking(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Hit>()?;
    m.add_class::<Track>()?;
    m.add_class::<Module>()?;
    m.add_class::<Event>()?;
    m.add_class::<TrackFollowing>()?;
    m.add_class::<GraphDFS>()?;
    m.add_class::<SearchByTripletTrie>()?;
    // m.add_class::<MCParticle>()?; dont need these might remove later
    // m.add_class::<Efficiency>()?;
    
    m.add_function(wrap_pyfunction!(validate_print, m)?)?;
    m.add_function(wrap_pyfunction!(validate_efficiency, m)?)?;
    m.add_function(wrap_pyfunction!(validate_to_json, m)?)?;
    m.add_function(wrap_pyfunction!(validate_to_json_nested, m)?)?;
    Ok(())
}

