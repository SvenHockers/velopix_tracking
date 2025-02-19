use pyo3::prelude::*;

// include custom impl
mod algorithms;
mod event_model;
use crate::event_model::event;
use crate::algorithms::track_following;
use crate::algorithms::graph_dfs;
use crate::algorithms::search_by_triplet_trie;

#[pymodule]
fn velopix_tracking(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!());
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("Custom velopix tracking")
    }
}
