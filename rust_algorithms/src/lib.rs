use pyo3::prelude::*;

// include custom impl
mod algorithms;
mod event_model;
use crate::event_model::event::Event;
use crate::algorithms::track_following::TrackFollowing;
use crate::algorithms::graph_dfs::GraphDFS;
use crate::algorithms::search_by_triplet_trie::SearchByTripletTrie;

#[pymodule]
fn velopix_tracking(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Event>()?;
    m.add_class::<TrackFollowing>()?;
    m.add_class::<GraphDFS>()?;
    m.add_class::<SearchByTripletTrie>()?;
    Ok(())
}