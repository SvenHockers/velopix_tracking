use pyo3::prelude::*;

// include custom impl
mod algorithms;
mod event_model;
use crate::event_model::event::Event;
use crate::algorithms::track_following::TrackFollowing;
// use crate::algorithms::graph_dfs;
// use crate::algorithms::search_by_triplet_trie;

#[pymodule]
fn velopix_tracking(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Event>()?;
    m.add_class::<TrackFollowing>()?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::{PyDict, PyList};

    // This function generates dummy event data
    fn create_dummy_event(py: Python) -> PyResult<event_model::event::Event> {
        let event_dict = PyDict::new(py);
        event_dict.set_item("description", "Upgrade 1")?;
        
        // Simulate 52 modules: 53 prefix sum values, assuming each module has 3 hits.
        let module_prefix_sum: Vec<usize> = (0..53).map(|i| i * 3).collect();
        event_dict.set_item("module_prefix_sum", module_prefix_sum)?;
        
        // Create a dummy montecarlo dictionary.
        let montecarlo = PyDict::new(py);
        montecarlo.set_item("description", vec!["key", "pid", "p", "pt", "eta", "phi"])?;
        // Create a sample particle as a Python list with explicit conversion.
        let particle = PyList::new(
            py,
            &[
                291.to_object(py),
                211.to_object(py),
                1390.to_object(py),
                220.to_object(py),
                (-3).to_object(py),
                (-3).to_object(py),
                0.to_object(py),
                0.to_object(py),
                1.to_object(py),
                0.to_object(py),
                0.to_object(py),
                0.to_object(py),
                0.to_object(py),
                0.to_object(py),
                1.0.to_object(py),
                PyList::new(
                    py,
                    &[
                        255.to_object(py),
                        310.to_object(py),
                        371.to_object(py),
                        427.to_object(py),
                        463.to_object(py),
                    ],
                )
                .to_object(py),
            ],
        );
        montecarlo.set_item("particles", PyList::new(py, &[particle]))?;
        event_dict.set_item("montecarlo", montecarlo)?;
        
        // Create coordinate arrays for x, y, and z.
        let total_hits = 156; // 52 modules * 3 hits each
        let x: Vec<f64> = (0..total_hits).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..total_hits).map(|i| (i as f64) * 2.0).collect();
        let z: Vec<f64> = (0..total_hits).map(|i| (i as f64) * 3.0).collect();
        event_dict.set_item("x", x)?;
        event_dict.set_item("y", y)?;
        event_dict.set_item("z", z)?;
        
        event_model::event::Event::new(py, event_dict)
    }

    #[test]
    fn test_track_following() {
        Python::with_gil(|py| {
            let event_instance = create_dummy_event(py)
                .expect("Failed to create dummy Event instance");

            // Instantiate the TrackFollowing solver with default parameters.
            let solver = algorithms::track_following::TrackFollowing::new(None, None, None); // use default values

            // Call the solver to obtain tracks.
            let tracks = solver.solve(&event_instance)
                .expect("Solver failed");

            // This can be an issue if dummy data is large
            println!("Found {} tracks.", tracks.len());
        });
    }
}

