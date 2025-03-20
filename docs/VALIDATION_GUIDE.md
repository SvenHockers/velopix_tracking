# **Validation Guide: Velopix Tracking Pipeline**  

## **Overview**  

The **Velopix Tracking Pipeline** includes a comprehensive **validation framework** to assess the accuracy and efficiency of track reconstruction. This guide explains the **validation metrics, implementation details, and how to use the validation functions**.  

The validation system is implemented in **Rust** for high-performance computations and is exposed to **Python** for usability.  

---

## **1. Validation Metrics**  

The validation process evaluates reconstructed tracks based on **several key metrics**:  

| **Metric**            | **Description** |
|-----------------------|----------------|
| **Reconstruction Efficiency** (`RecoEff`) | Fraction of true tracks successfully reconstructed. |
| **Ghost Rate**        | Fraction of falsely reconstructed tracks (noise or incorrect matches). |
| **Clone Rate**        | Fraction of duplicate reconstructed tracks. |
| **Hit Efficiency**    | Fraction of true particle hits used in reconstructed tracks. |
| **Track Purity**      | Fraction of hits in a track that originate from a single true particle. |

Each event undergoes **multiple category-based evaluations** (e.g., **long tracks, velo tracks, high-energy tracks**).  

---

## **2. Validation System Components**  

The validation framework consists of **several core modules**:  

### **2.1. Efficiency Computation (`efficientcy.rs`)**  

This module computes **per-category efficiency metrics** based on **Monte Carlo truth information**.  

#### **Key Attributes**  
- `n_particles`: Total number of true particles in the event.  
- `n_reco`: Number of reconstructed tracks.  
- `n_clones`: Number of duplicate tracks.  
- `recoeff_t`: Overall track reconstruction efficiency.  
- `purity_t`: Average purity of reconstructed tracks.  
- `avg_hiteff`: Average fraction of hits correctly assigned to tracks.  

#### **Example Usage**  
```python
from velopix_tracking import validate_efficiency

efficiency = validate_efficiency(py_events, py_tracks, particle_type="long")
print(f"Track Reconstruction Efficiency: {efficiency:.2f}")
```

---

### **2.2. Event Representation (`event.rs`)**  

The `ValidatorEvent` class encapsulates **hit-level and track-level data** for an event, including:  
- **Raw detector hits**  
- **Module-level hit information**  
- **Monte Carlo truth particles**  

#### **Example Usage**  
```python
from velopix_tracking import ValidatorEvent

event = ValidatorEvent(module_prefix_sum, hit_xs, hit_ys, hit_zs, hits)
hit = event.get_hit(0)  # Retrieve first hit
```

---

### **2.3. Track-Particle Matching (`helper.rs`)**  

The helper functions establish **track-particle associations** and compute **track purity**:  

| **Function**      | **Description** |
|-------------------|----------------|
| `hit_purity()`   | Computes purity scores for tracks and their associated Monte Carlo particles. |
| `ghost_rate()`   | Determines the proportion of tracks that do not correspond to real particles. |
| `clones()`       | Identifies duplicate tracks assigned to the same true particle. |
| `hit_efficiency()` | Measures the fraction of true particle hits assigned to reconstructed tracks. |

#### **Example Usage**  
```python
from velopix_tracking import ghost_rate, hit_efficiency

ghost_fraction, ghost_count = ghost_rate(track_to_particle_map)
hit_eff = hit_efficiency(track_to_particle_map, event)
```

---

## **3. Validation Methods**  

The validation framework provides **several high-level functions** for evaluating reconstructed tracks.  

### **3.1. Print Validation Summary (`validate_print`)**  

Outputs a **detailed validation summary**, including **track efficiency, ghost rate, and clone rate**.  

```python
from velopix_tracking import validate_print

validate_print(py_events, py_tracks)
```

#### **Example Output**  
```
148 tracks including 8 ghosts (5.4%). Event average: 5.4%
velo : 126 from 134 (94.0%) 3 clones (2.38%) purity: (98.83%) hitEff: (93.89%)
long : 22 from 22 (100.0%) 1 clones (4.55%) purity: (99.52%) hitEff: (93.80%)
```

---

### **3.2. JSON Validation Output (`validate_to_json`)**  

Generates **structured JSON output** containing **detailed efficiency metrics**.  

```python
from velopix_tracking import validate_to_json

validation_results = validate_to_json(py_events, py_tracks, verbose=True)
print(validation_results)
```

#### **Example JSON Output**  
```json
{
    "total_tracks": 148,
    "total_ghosts": 8,
    "overall_ghost_rate": 5.4,
    "event_avg_ghost_rate": 5.4,
    "categories": [
        {
            "label": "velo",
            "n_reco": 126,
            "n_particles": 134,
            "recoeffT": 94.0,
            "avg_recoeff": 94.0,
            "n_clones": 3,
            "clone_percentage": 2.38,
            "purityT": 98.83,
            "avg_purity": 98.83,
            "avg_hiteff": 93.89
        },
        {
            "label": "long",
            "n_reco": 22,
            "n_particles": 22,
            "recoeffT": 100.0,
            "avg_recoeff": 100.0,
            "n_clones": 1,
            "clone_percentage": 4.55,
            "purityT": 99.52,
            "avg_purity": 99.52,
            "avg_hiteff": 93.80
        }
    ]
}
```

---

### **3.3. Nested JSON Validation (`validate_to_json_nested`)**  

This function provides **per-event validation summaries** in a structured format.  

```python
from velopix_tracking import validate_to_json_nested

validation_results = validate_to_json_nested(py_events, py_tracks, verbose=True)
```

#### **Example Nested JSON Output**  
```json
{
    "total_tracks": 148,
    "total_ghosts": 8,
    "overall_ghost_rate": 5.4,
    "event_avg_ghost_rate": 5.4,
    "events": {
        "1": [...], 
        "2": [...], 
        "3": [...]
    }
}
```

---

## **4. Summary of Validation Features**  

| Feature | Description |
|---------|-------------|
| **Efficiency Calculation** | Evaluates track reconstruction accuracy per particle category. |
| **Ghost & Clone Detection** | Identifies fake and duplicate tracks. |
| **Track-Purity Analysis** | Computes how well reconstructed tracks match real particles. |
| **Structured Validation Output** | Generates detailed JSON validation reports. |
| **Optimized Performance** | Implemented in Rust for high-speed evaluation. |

---

## **5. Additional Resources**  

For more in-depth details, refer to:   
- [Developer Guide](DEVELOPER_GUIDE.md)  

