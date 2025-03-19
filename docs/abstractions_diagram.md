# Implementation and Abstraction



# Overview : how does everything interact
```mermaid 
classDiagram
class Event_Model_Module {<<module>>} 
class Velopix_Tracking_Module {<<module>>} 
class Core_Wrappers_Module {<<module>>} 
class Pipelines_Module {<<module>>} 
class ExampleOptimizer {<<Optimizer Implementation>>
    + Optimization Algorithm
    + __init__()
    + start()
    + is_finished()
    + objective_func()}

Event_Model_Module --> Velopix_Tracking_Module : interacts with
Event_Model_Module --> Core_Wrappers_Module : uses
Event_Model_Module --> Pipelines_Module : creates events for

Velopix_Tracking_Module --> Core_Wrappers_Module : uses metrics from
Velopix_Tracking_Module *-- Pipelines_Module : creates pipelines for

Core_Wrappers_Module <|.. ExampleOptimizer : Implements Optimizer
Pipelines_Module ..> ExampleOptimizer : Selects Reconstruction Algorithm
```

## Event Model : Class Diagram

```mermaid 
classDiagram
namespace Event_Model {
    class event {
        +__init__(json_data)
        -number_of_modules : int = 52
        -description : str
        -montecarlo
        -module_prefix_sum
        -number_of_hits : int
        -module_zs : list
        -hits : list
        -modules : list
    }
    
    class track {
        <<internal>>
        +__init__(hits)
        +add_hit(hit)
        +__repr__() : str
        +__iter__()
        +__eq__(other)
        +__ne__(other)
        +__hash__()
        -hits : list
        -missed_last_module : bool
        -missed_penultimate_module : bool
    }
    
    class hit {
        <<internal>>
        +__init__(x, y, z, hit_id, module=-1, t=0, with_t=False)
        +__getitem__(index)
        +__repr__() : str
        +__eq__(other)
        +__hash__()
        -x
        -y
        -z
        -t
        -id
        -module_number
        -with_t
    }
    
    class module {
        <<internal>>
        +__init__(module_number, z, start_hit, number_of_hits, hits)
        +__iter__()
        +__repr__() : str
        +hits() : list
        -module_number : int
        -z
        -hit_start_index : int
        -hit_end_index : int
        -__global_hits : list
    }
}

event "1" *-- "*" hit : creates
event "1" *-- "*" module : creates
track "1" *-- "*" hit : contains
module "1" o-- "*" hit : aggregates

event --() velopix_tracking
track --() velopix_tracking
```

## Velopix Tracking Algorithms : Class Diagram 

```mermaid 
classDiagram
namespace Velopix_Tracking {    
    class TrackFollowing {
        <<reconstruction algorithm>>
        +__init__(*args*)
        +solve_parallel(events: List[Event]) : List[Track]
    }
    
    class GraphDFS {
        <<reconstruction algorithm>>
        +__init__(*args*)
        +solve_parallel(events: List[Event]) : List[Track]
    }
    
    class SearchByTripletTrie {
        <<reconstruction algorithm>>
        +__init__(*args*)
        +solve_parallel(events: List[Event]) : List[Track]
    }
    
    class validate_to_json_nested {
        <<utility>>
        +(json_events: List[dict], tracks: List[Track], verbose: bool) : dict
    }
    
    class validate_to_json {
        <<utility>>
        +(json_events: List[dict], tracks: List[Track], verbose: bool) : dict
    }
}

%% External stub definitions for classes used in relationships
class Pipeline_TrackFollowing {<<external>>}
class Pipeline_GraphDFS {<<external>>}
class Pipeline_SearchByTripletTrie {<<external>>}

TrackFollowing --|> Pipeline_TrackFollowing : uses
GraphDFS --|> Pipeline_GraphDFS : uses
SearchByTripletTrie --|> Pipeline_SearchByTripletTrie : uses
```



## Core Wrappers : Class Diagram

```mermaid
classDiagram
namespace Core_Wrappers {
    class ReconstructionAlgorithms {
        <<enumeration>>
        +TRACK_FOLLOWING
        +GRAPH_DFS
        +SEARCH_BY_TRIPLET_TRIE
        +get_config()
        +_bounds()
    }
    
    class EventMetricsCalculator {
        -validation_results
        -df_events
        +__init__(validation_results)
        +compute_aggregations()
        +flatten_aggregations(agg_df)
        +compute_average_metric(metrics, col, stat)
        +get_metric(metric, stat)
    }
    
    class optimiserBase {
        <<abstract>>
        -objective : str
        -best_score : float
        -best_config : dict
        -_algorithm
        -run
        +__init__(Objective)
        +start(algorithm)
        +next_pMap()
        +add_run(results)
        +get_optimised_pMap()
        +get_run_data()
        +init() <<abstract>>
        +next() <<abstract>>
        +is_finished() <<abstract>>
        +objective_func() <<abstract>>
        +event_objective()
        +intra_event_objective()
        +validate_config(config, schema)
    }
    
    class PipelineBase {
        <<abstract>>
        -name : str
        -json_events : List~Dict~
        -nested : bool
        -parameters : List~Dict~
        -events : List~Event~
        -results : List~Dict~
        +__init__(events, intra_node, parameter_map)
        +run(overwrite)
        +optimise_parameters(Optimiser, max_runs)
        +set_pMap(pMap)
        +get_results()
        +calculate_db_estimate()
        +print_validation()
        +generate_database(output_directory, overwrite)
        +generate_and_get_database()
        +model(map) <<abstract>>
    }
}

%% Stub definitions for classes referenced outside Core_Wrappers
class event {<<external>>}
namespace PipelineImplementation {
class Pipeline_TrackFollowing {<<external>>}
class Pipeline_GraphDFS {<<external>>}
class Pipeline_SearchByTripletTrie {<<external>>}}
class ExampleOptimizer {<<external>>
    + Optimization Algorithm
    + __init__()
    + start()
    + is_finished()
    + objective_func()}
class Validation {<<external>>
+validation_to_json()
+validation_to_json_nested()}
%% Relationships

optimiserBase <|-- ExampleOptimizer : Implements
optimiserBase ..> EventMetricsCalculator : uses
optimiserBase ..> ReconstructionAlgorithms : uses
PipelineBase ..> optimiserBase : uses
PipelineBase ..> event : uses
PipelineBase <-- Pipeline_TrackFollowing : sets reconstruction algorithm
PipelineBase <-- Pipeline_GraphDFS : sets reconstruction algorithm
PipelineBase <-- Pipeline_SearchByTripletTrie : sets reconstruction algorithm

EventMetricsCalculator ..> Validation : uses
```

---

[Go Back](../readme.md)