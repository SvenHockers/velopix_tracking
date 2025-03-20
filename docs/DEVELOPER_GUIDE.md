# **Velopix Tracking Pipeline Developer Guide**  

## **Overview**  

The **Velopix Tracking Pipeline** provides a high-performance framework for **particle trajectory reconstruction** using event data from the **LHCb detector at CERN**.  

This documentation covers:  
- **Core Data Structures** – Representing detector hits, modules, and events.  
- **Tracking Algorithms** – Implementation of **Track-Following, GraphDFS, and SearchByTripletTrie**.  
- **Pipeline Framework** – Abstraction layers for **tracking, validation, and optimization**.  
- **Hyperparameter Optimization** – Automated parameter tuning using `optimiserBase`.  

---

## **1. Algorithm Schema (`ReconstructionAlgorithms`)**  

The `ReconstructionAlgorithms` **Enum** standardizes the available **tracking methods** and their parameters.  

### **1.1. Available Tracking Methods**  

| Algorithm                  | Description |
|----------------------------|-------------|
| `TRACK_FOLLOWING`          | Sequential hit matching along expected particle trajectories. |
| `GRAPH_DFS`                | Graph-based **Depth-First Search** for track reconstruction. |
| `SEARCH_BY_TRIPLET_TRIE`   | Trie-based **triplet searching** for high-density events. |

### **1.2. Standardized Parameter Definitions**  

Each tracking method has its own **parameter set** with predefined **types and constraints**.  

```python
from enum import Enum

class ReconstructionAlgorithms(Enum):
    TRACK_FOLLOWING = {
        "x_slope": (float, None),
        "y_slope": (float, None),
        "x_tol": (float, None),
        "y_tol": (float, None),
        "scatter": (float, None),
    }
```

#### **Example: Retrieving Algorithm Parameters**  
```python
config = ReconstructionAlgorithms.GRAPH_DFS.get_config()
print(config)  # Returns default parameter structure
```

#### **Parameter Bounds for Optimization**  
The `_bounds()` method ensures that **optimization functions** adhere to valid ranges.  

```python
bounds = ReconstructionAlgorithms.GRAPH_DFS._bounds()
print(bounds)  
# Returns the allowed min/max values for each parameter
```

---

## **2. Data Structures for Tracking**  

Velopix defines core data structures for **handling detector hits, modules, and reconstructed tracks**.  

### **2.1. Hit (Single Measurement in the Detector)**  
A **Hit** represents a **single recorded interaction** in the detector.  

```python
hit = Hit(x=12.3, y=-5.8, z=302.1, hit_id=7, module=3)
print(hit)
```

---

### **2.2. Module (Detector Layer)**  
A **Module** represents a **layer in the detector** where multiple hits are recorded.  

```python
module = Module(module_number=3, z=302.1, hit_start_index=0, hit_end_index=10, global_hits=[hit])
print(module.hits())  
```

---

### **2.3. Event (Full Collision Data)**  
An **Event** contains **all hits and modules** recorded during a collision.  

```python
event = Event(json_data=event_json)
print(len(event.hits))  
```

---

## **3. Tracking Algorithms**  

Velopix implements **three distinct algorithms** for track reconstruction.  

### **3.1. Track-Following Algorithm**  
```python
tracker = TrackFollowing(max_slopes=(0.2, 0.2), min_track_length=5)
tracks = tracker.solve(event)
```

---

### **3.2. GraphDFS Algorithm**  
```python
tracker = GraphDFS(allowed_skip_modules=2, clone_ghost_killing=True)
tracks = tracker.solve(event)
```

---

### **3.3. SearchByTripletTrie Algorithm**  
```python
tracker = SearchByTripletTrie(max_scatter=0.3)
tracks = tracker.solve(event)
```

---

## **4. Event Metrics Calculation (`EventMetricsCalculator`)**  

The `EventMetricsCalculator` **aggregates validation results** and computes **statistics** for performance evaluation.  

```python
calculator = EventMetricsCalculator(validation_results)
metrics = calculator.compute_aggregations()
print(metrics)
```

### **Example: Retrieve Standard Deviation of Clone Percentage**  
```python
clone_std = calculator.get_metric(metric="clone_percentage", stat="std")
print(clone_std)
```

---

## **5. Optimization Framework (`optimiserBase`)**  

The **optimiserBase** class enables **automated hyperparameter tuning** for reconstruction algorithms.  

### **5.1. Example: Running a Parameter Optimization**  

```python
from parameter_optimisers import optimiserBase

optimiser = optimiserBase(Objective="min")
best_params = optimiser.start(ReconstructionAlgorithms.GRAPH_DFS)

while not optimiser.is_finished():
    pipeline.set_pMap([optimiser.next_pMap()])
    pipeline.run(overwrite=True)
    optimiser.add_run(pipeline.get_results()[-1])
```

---

## **6. Tracking Pipeline Framework (`PipelineBase`)**  

The **PipelineBase** class abstracts **event processing**, tracking execution, and validation.  

### **6.1. Running a Tracking Pipeline**  

```python
pipeline = Pipeline_GraphDFS(events=[event_json], intra_node=True)
pipeline.run(overwrite=True)

# Retrieve results
results = pipeline.get_results()
pipeline.generate_database("output_directory", overwrite=True)
```

---

## **7. Summary of Features**  

| Feature | Description |
|---------|-------------|
| **Standardized Parameter Schema** | Ensures consistency in algorithm configurations. |
| **Automated Hyperparameter Tuning** | Optimizes parameters via `optimiserBase`. |
| **Modular Pipeline Framework** | Extends `PipelineBase` for flexibility. |
| **Validation & Performance Metrics** | Computes detailed statistics with `EventMetricsCalculator`. |
| **Rust-Optimized Performance** | Uses high-speed Rust-based tracking implementations. |

---

## **8. Additional Resources**  

For more in-depth details, refer to:  
- [Track Reconstruction Documentation](../readme.md)  
- [Performance Validation](./docs/VALIDATION_GUIDE.md)  

[🔙 Return to README](../readme.md)  

---

## **Changelog & Improvements**  

✅ **Added Algorithm Schema (`ReconstructionAlgorithms`)** – Standardized tracking configurations.  
✅ **Integrated `EventMetricsCalculator`** – Performance metric computation & analysis.  
✅ **Refined `optimiserBase`** – Automated **hyperparameter tuning** for track reconstruction.  
✅ **Enhanced `PipelineBase`** – Improved **pipeline management & validation integration**.  

This **developer guide** now provides a **comprehensive reference** 


for using and extending the **Velopix Tracking Pipeline**. 🚀  
