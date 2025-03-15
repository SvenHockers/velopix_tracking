# Velopix Tracking Pipeline Documentation

## Overview
This documentation provides an overview of the `velopix_pipeline` and `validation_to_datasets` modules, which are used for tracking event data, performing validations, and saving results in structured formats. These modules work together to provide an efficient way to process event data using different tracking models.

## `velopix_pipeline` Module

The `velopix_pipeline` module defines an abstract pipeline class and its concrete implementations for different tracking algorithms.

### `PipelineBase`
This is an abstract base class for tracking pipelines.

#### **Attributes:**
- `json_events`: List of raw event data in JSON format.
- `nested`: Boolean indicating whether nested validation is used.
- `parameters`: List of parameter dictionaries.
- `events`: List of parsed `Event` objects.
- `results`: List to store processing results.

#### **Methods:**
- `model(map) -> Union[TrackFollowing, GraphDFS, SearchByTripletTrie]`
  - Abstract method to be implemented by subclasses, returning an appropriate tracking model.

- `run(overwrite: bool) -> None`
  - Executes the pipeline, processes events, and stores results.

- `get_results() -> List[Dict[str, Any]]`
  - Returns the stored results from the tracking process.

- `calculate_db_estimate() -> None`
  - Estimates the storage requirements of the generated database.

- `print_validation() -> None`
  - Prints validation results.

- `generate_database(output_directory: str, overwrite: bool) -> None`
  - Saves validation results to the specified output directory.

- `generate_and_get_database() -> Tuple[pd.DataFrame, pd.DataFrame, Optional[pd.DataFrame]]`
  - Returns the validation results as Pandas DataFrames.

### Concrete Implementations

#### `Pipeline_TrackFollowing`
Implements the `TrackFollowing` tracking algorithm.

**Method:**
- `model(map: Dict[str, Any]) -> TrackFollowing`

#### `Pipeline_GraphDFS`
Implements the `GraphDFS` tracking algorithm.

**Method:**
- `model(map: Dict[str, Any]) -> GraphDFS`

#### `Pipeline_SearchByTripletTrie`
Implements the `SearchByTripletTrie` tracking algorithm.

**Method:**
- `model(map: Dict[str, Any]) -> SearchByTripletTrie`

## `validation_to_datasets` Module

The `validation_to_datasets` module processes validation results and saves them as structured datasets.

### **Functions:**

#### `save_to_file(results: List[Dict[str, Any]], directory: str, output_func: str = "output_aggregates", overwrite: bool = False) -> None`
Saves processed validation results to CSV files.

#### `output_aggregates(results: List[Dict[str, Any]]) -> Tuple[pd.DataFrame, pd.DataFrame]`
Processes aggregated validation results and returns two DataFrames:
- `overall_df`: Summary per run.
- `category_df`: Summary per category per run.

#### `output_distributions(results: List[Dict[str, Any]]) -> Tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame]`
Processes validation results using nested validation and returns three DataFrames:
- `overall_df`: Summary per run.
- `category_df`: Summary per category per run.
- `distribution_df`: Event-level statistical distributions.

## Usage Example

```python
from velopix_pipeline import Pipeline_GraphDFS

# Example event data and parameters
events = [{...}, {...}] # List of JSON object (ie Events)
parameters = [{"x_slope": 0.1, "y_slope": 0.2, "scatter": 0.3}]

# Instantiate and run the pipeline
pipeline = Pipeline_GraphDFS(events, parameters, intra_node=True)
pipeline.run(overwrite=True)

# Get and save results
results = pipeline.get_results()
pipeline.generate_database("output_directory", overwrite=True)
```
---
[Go Back](./velopix_tracking.md)