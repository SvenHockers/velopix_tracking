use pyo3::prelude::*;

// include custom functions
pub mod event_model;
mod algorithms;

// remove this once dev is done
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

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
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
