from abc import ABC, abstractmethod
import time
import pandas as pd
from validation_to_datasets import *
from velopix_tracking import Event, TrackFollowing, GraphDFS, SearchByTripletTrie, \
                            validate_print, validate_to_json_nested, validate_to_json

from typing import Any, Dict, List, Optional, Union, Tuple

class PipelineBase(ABC):
    def __init__(self, events: List[Dict[str, Any]]) -> None:
        self.json_events = events

        self.events = []
        for event in events:
            self.events.append(Event(event))

    @abstractmethod
    def model(map) -> Union[TrackFollowing, GraphDFS, SearchByTripletTrie]:
        pass

    def run(self, parameter_map: List[Dict[str, Any]], intra_node: bool) -> None:
        # here I should include a warning if self.results != empty break to prevent loss of data
        self.nested = intra_node
        self.results =  []
        for pMap in parameter_map:
            model = self.model(pMap)
            tstart = time.strptime()
            self.tracks = model.solve_parallel(self.events)
            runtime = time.strptime() - tstart 
            if intra_node:
                valMap = validate_to_json_nested(self.json_events, self.tracks)
            else:
                valMap = validate_to_json(self.json_events, self.tracks)
            valMap["inference_time"] = runtime
            valMap["parameters"] = {
                "max_slope": (pMap.get("x_slope"), pMap.get("y_slope")),
                "max_tol": (pMap.get("x_slope"), pMap.get("y_slope")),
                "scatter": pMap.get("scatter")
            }
            self.results.append(valMap)

    def get_results(self) -> List[Dict[str, Any]]:
        return self.results
    
    # note this print func is computationally heavy
    def print_validation(self) -> None:
        validate_print(self.json_events, self.tracks)

    def generate_database(self, output_directory: str, overwrite: bool) -> None:
        func = "output_distributions" if self.nested else "output_aggregates"
        save_to_file(results=self.results, directory=output_directory, output_func=func, overwrite=overwrite)

    def generate_and_get_database(self) -> Tuple[pd.DataFrame, pd.DataFrame, Optional[pd.DataFrame]]:
        sfunc = "output_distributions" if self.nested else "output_aggregates"
        func = getattr(sfunc, None)
        if not callable(func):
            raise ValueError(f"Output function '{sfunc}' not found.")
        
        if sfunc == "output_aggregates":
            return func(self.results)
        else:
            return func(self.results)
        
class Pipeline_TrackFollowing(PipelineBase):
    def model(map: Dict[str, Any]) -> TrackFollowing:
        return TrackFollowing(
            max_slopes=(map.get("x_slope"), map.get("y_slope")),
            max_tolerance=(map.get("x_tol"), map.get("y_tol")),
            max_scatter=map.get("scatter")
        )
    
class Pipeline_GraphDFS(PipelineBase):
    def model(map: Dict[str, Any]) -> GraphDFS:
        return GraphDFS(
            max_slopes=(map.get("x_slope"), map.get("y_slope")),
            max_tolerance=(map.get("x_tol"), map.get("y_tol")),
            max_scatter=map.get("scatter"),
            minimum_root_weight=map.get("minimum_root_weight"),
            weight_assignment_iterations=map.get("weight_assignment_iterations"),
            allowed_skip_modules=map.get("allowed_skip_modules"),
            allow_cross_track=map.get("allow_cross_track"),
            clone_ghost_killing=map.get("clone_ghost_killing")
        )
    
class Pipeline_SearchByTripletTrie(PipelineBase):
    def model(map: Dict[str, Any]) -> SearchByTripletTrie:
        return SearchByTripletTrie(
            max_scatter=map.get("scatter"),
            min_track_length=map.get("min_track_length"),
            min_strong_track_length=map.get("min_strong_track_length"),
            allowed_missed_modules=map.get("allowed_missed_modules")
        )