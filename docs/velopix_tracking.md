# Velopix Tracking Documentation

This documentation provides an overview of the **velopix_tracking** package API and guides you on how to use its classes and functions. The package is designed for reconstructing particle tracks from detector events using data structures that represent hits, tracks, modules, and events. It offers several reconstruction algorithms and validation methods to assess the quality of the reconstruction.

## Table of Contents
- [Overview](#overview)
- [Installation](#installation)
- [API Reference](#api-reference)
  - [Virtual Environment Classes](#virtual-environment-classes)
  - [Reconstruction Algorithms](#reconstruction-algorithms)
  - [Validation Methods](#validation-methods)
- [Usage Examples](#usage-examples)
- [Improvement Suggestions](#improvement-suggestions)

---

## Overview

The **velopix_tracking** package is focused on reconstructing particle tracks using real detector data. It contains:

- **Virtual Environment Classes**:  
  - **Hit**: Represents a single detector hit.
  - **Track**: A reconstructed track composed of multiple hits.
  - **Module**: A detector module grouping hits.
  - **Event**: An entire event including hits and modules.

- **Reconstruction Algorithms**:  
  Three algorithms for track reconstruction:
  - **TrackFollowing**: A sequential method based on geometric constraints.
  - **GraphDFS**: Uses depth-first search over a graph representation of hit segments.
  - **SearchByTripletTrie**: An advanced algorithm that merges modules and uses a trie structure for hit triplets.

- **Validation Methods**:  
  Functions to validate and benchmark the reconstruction quality.

---

## Installation

Please refer to [installation guide](./INSTALLATION.md) for the installation scripts. Below a description of manual installation is described:

First ensure you've installed `Rust` then install maturin:

```bash
pip install maturin
```
  
Move to the rust directory and build the python package:
```bash 
cd rust_codebase
maturin build --release
```

---

## API Reference

### Virtual Environment Classes

#### `Hit`
Represents a detector hit with spatial and timing information.

**Attributes:**
- `id`: Unique identifier.
- `x`, `y`, `z`: Spatial coordinates.
- `t`: Time value.
- `module_number`: Detector module number.
- `with_t`: Flag indicating if time data is provided.

**Constructor:**
```python
Hit(x: float, y: float, z: float, hit_id: int, module: int = None, t: float = None, with_t: bool = None)
```

**Example:**
```python
from velopix_tracking import Hit

hit = Hit(9.18, -30.509, -288.08, hit_id=0, module=0, t=12.3, with_t=True)
print(hit)
```
```
#0 module 0 {9.18, -30.509, -288.08, 12.3}
```

---

#### `Track`
Represents a particle track, which is a collection of hits.

**Attributes:**
- `hits`: List of `Hit` objects.
- `missed_last_module`: Indicates if the last module was missed.
- `missed_penultimate_module`: Indicates if the second-to-last module was missed.

**Constructor:**
```python
Track(hits: List[Hit])
```

**Methods:**
- `add_hit(hit: Hit) -> None`: Adds a hit to the track.
- `__repr__() -> str`: Returns a string representation.

**Example:**
```python
from velopix_tracking import Track, Hit

hit1 = Hit(9.18, -30.509, -288.08, hit_id=0, module=0)
hit2 = Hit(10.0, -29.0, -287.5, hit_id=1, module=0)
track = Track([hit1])
track.add_hit(hit2)
print(track)
```
```
Track with 2 hits: [Hit { id: 0, x: 9.18, y: -30.509, z: -288.08, t: 0.0, module_number: 0, with_t: false }, Hit { id: 1, x: 10.0, y: -29.0, z: -287.5, t: 0.0, module_number: 0, with_t: false }]
```

---

### Reconstruction Algorithms

Each algorithm provides methods to extract tracks from an event.

#### `TrackFollowing`
A sequential algorithm that matches hits based on geometric constraints.
For detail overview of how this algorithm works view the [TrackFollowing documentation](./ALGO_TrackFollowing.md).

**Methods:**
- `solve(event: Event) -> List[Track]`: Processes a single event.
- `solve_parallel(events: List[Event]) -> List[Track]`: Processes multiple events in parallel.

**Example:**
```python
from velopix_tracking import TrackFollowing, Event
import json

with open("event.json") as f:
    event_data = json.load(f)

event = Event(event_data)
tf = TrackFollowing(max_slopes=(0.5, 1.0), min_track_length=3)
tracks = tf.solve(event)
```

#### `GraphDFS`
A graph-based depth-first search algorithm that builds a directed graph of hits and searches for track candidates.
For a detailed overview, view the [GraphDFS documentation](./ALGO_DFS.md).

**Methods:**
- `solve(event: Event) -> List[Track]`: Processes a single event.
- `solve_parallel(events: List[Event]) -> List[Track]`: Processes multiple events in parallel.

**Example:**
```python
from velopix_tracking import GraphDFS, Event
import json

with open("event.json") as f:
    event_data = json.load(f)

event = Event(event_data)
graph_dfs = GraphDFS(max_scatter=0.2, minimum_root_weight=2)
tracks = graph_dfs.solve(event)
```

#### `SearchByTripletTrie`
An advanced algorithm that builds a trie of hit triplets and reconstructs tracks based on scatter constraints.
For a detailed overview, view the [SearchByTripletTrie documentation](ALGO_SearchByTripletTree.md).

**Methods:**
- `solve(event: Event) -> List[Track]`: Processes a single event.
- `solve_parallel(events: List[Event]) -> List[Track]`: Processes multiple events in parallel.

**Example:**
```python
from velopix_tracking import SearchByTripletTrie, Event
import json

with open("event.json") as f:
    event_data = json.load(f)

event = Event(event_data)
triplet_search = SearchByTripletTrie(max_scatter=0.1, min_track_length=4)
tracks = triplet_search.solve(event)
```

---

### Validation Methods

Use these functions to evaluate the track reconstruction performance.

**Example:**
```python
from velopix_tracking import validate_print, validate_efficiency, validate_to_json
import json

# Load event data
with open("event.json") as f:
    event_data = json.load(f)

# Assuming 'tracks' is a list of Track objects
validate_print([event_data], [tracks])
efficiency = validate_efficiency([event_data], [tracks], particle_type="velo")
json_results = validate_to_json([event_data], [tracks], verbose=True)

print("Efficiency:", efficiency)
print("Validation JSON:", json_results)
```

---

## Usage Examples

### Example 1: Reconstructing Tracks from a Single Event

```python
import json
from velopix_tracking import Event, TrackFollowing

# Load event data from a JSON file
with open("event.json") as f:
    event_data = json.load(f)

# Create an Event object
event = Event(event_data)

# Initialize the TrackFollowing algorithm
tf = TrackFollowing(max_slopes=(0.5, 1.0), min_track_length=3)

# Reconstruct tracks from the event
tracks = tf.solve(event)

# Display the reconstructed tracks
print("Reconstructed Tracks:")
for track in tracks:
    print(track)
```

---

## Improvement Suggestions

- [x] **Abstraction Wrappers:** Build logic wrappers for better ease of use ([link](./wrappers.md))
- [ ] **GPU Acceleration:** Configure GPU compatabillity for optimal performace.
- [ ] **Enhanced Examples:** Include error handling for missing or malformed data.
- [ ] **Parameter Details:** Provide more details on each constructor parameter.
- [ ] **Visual Diagrams:** Add flowcharts illustrating the reconstruction process.
- [ ] **Extended Validation Output:** Include sample outputs from validation functions.
- [ ] **Integration Guidelines:** Offer recommendations on integrating **velopix_tracking** into larger projects.

[Go Back](../readme.md)
