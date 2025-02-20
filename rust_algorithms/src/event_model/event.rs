use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use std::collections::HashSet;
use ordered_float::OrderedFloat;
use crate::event_model::hit::Hit;
use crate::event_model::module::Module;

#[pyclass]
#[derive(Clone)]
pub struct Event {
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub montecarlo: PyObject,
    #[pyo3(get)]
    pub module_prefix_sum: Vec<usize>,
    #[pyo3(get)]
    pub number_of_hits: usize,
    /// Converted to a Vec of f64 values for each module.
    #[pyo3(get)]
    pub module_zs: Vec<Vec<f64>>,
    #[pyo3(get)]
    pub hits: Vec<Hit>,
    #[pyo3(get)]
    pub modules: Vec<Module>,
}

#[pymethods]
impl Event {
    #[new]
    pub fn new(_py: Python, json_data: &PyAny) -> PyResult<Self> {
        // Convert the input into a Python dictionary.
        let dict: &PyDict = json_data.downcast()?;
        
        // Extract required fields.
        let description: String = dict.get_item("description").unwrap().extract()?;
        let montecarlo: PyObject = dict.get_item("montecarlo").unwrap().into();
        let module_prefix_sum: Vec<usize> = dict.get_item("module_prefix_sum").unwrap().extract()?;
        
        // Set number_of_modules and compute number_of_hits.
        let number_of_modules = 52;
        let number_of_hits = module_prefix_sum[number_of_modules];
        
        // Check if "t" is provided.
        let with_t = dict.contains("t")?;
        
        // Extract coordinate lists.
        let x: Vec<f64> = dict.get_item("x").unwrap().extract()?;
        let y: Vec<f64> = dict.get_item("y").unwrap().extract()?;
        let z: Vec<f64> = dict.get_item("z").unwrap().extract()?;
        let t: Option<Vec<f64>> = if with_t {
            Some(dict.get_item("t").unwrap().extract()?)
        } else {
            None
        };

        // Prepare a vector of HashSet to collect unique z values per module.
        let mut module_zs_sets: Vec<HashSet<OrderedFloat<f64>>> = vec![HashSet::new(); number_of_modules];
        let mut hits: Vec<Hit> = Vec::with_capacity(number_of_hits);

        // Process each module's hits.
        for m in 0..number_of_modules {
            let start = module_prefix_sum[m];
            let end = module_prefix_sum[m + 1];
            for i in start..end {
                // Create a new Hit instance.
                let hit_obj = if with_t {
                    Hit::new(
                        x[i],
                        y[i],
                        z[i],
                        i as u32,
                        Some(m as i32),
                        Some(t.as_ref().unwrap()[i]),
                        Some(true),
                    )
                } else {
                    Hit::new(
                        x[i],
                        y[i],
                        z[i],
                        i as u32,
                        Some(m as i32),
                        None,
                        None,
                    )
                };
                // Insert the z value into the set.
                module_zs_sets[m].insert(OrderedFloat(z[i]));
                hits.push(hit_obj);
            }
        }

        // Convert each module's HashSet into a Vec<f64>.
        let module_zs: Vec<Vec<f64>> = module_zs_sets.into_iter()
            .map(|set| set.into_iter().map(|of| of.into_inner()).collect())
            .collect();

        // Create modules.
        // Note: We pass the absolute end index (module_prefix_sum[m + 1]) instead of the hit count.
        let modules: Vec<Module> = (0..number_of_modules)
            .map(|m| {
                let start = module_prefix_sum[m];
                let end = module_prefix_sum[m + 1];
                Module::new(m as u32, module_zs[m].clone(), start, end, hits.clone())
            })
            .collect::<PyResult<Vec<_>>>()?;
        
        // Return the constructed Event instance.
        Ok(Event {
            description,
            montecarlo,
            module_prefix_sum,
            number_of_hits,
            module_zs,
            hits,
            modules,
        })
    }
}
