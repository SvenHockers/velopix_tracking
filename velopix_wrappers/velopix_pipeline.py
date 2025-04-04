from abc import ABC, abstractmethod
import time
import pandas as pd
from .validation_to_datasets import *
from velopix import Event, TrackFollowing, GraphDFS, SearchByTripletTrie, \
                            validate_print, validate_to_json_nested, validate_to_json
from .parameter_optimisers import optimiserBase
from .algorithm_schema import ReconstructionAlgorithms
from typing import Any, Dict, List, Optional, Union, Tuple
from tqdm import tqdm
import warnings

class PipelineBase(ABC):
    def __init__(self, events: List[Dict[str, Any]], intra_node: bool, parameter_map: List[Dict[str, Any]] = None) -> None:
        self.name = "Base"
        self.json_events = events
        self.nested = intra_node
        self.parameters = parameter_map
        self.events = []
        for event in events:
            self.events.append(Event(event))

    @abstractmethod
    def model(map) -> Union[TrackFollowing, GraphDFS, SearchByTripletTrie]: pass

    def run(self, overwrite: bool) -> None:
        # here I should include a warning if self.results != empty break to prevent loss of data
        if hasattr(self, "results") and not overwrite:
            warnings.warn("Overwriting results. This will cause a loss of data!", UserWarning)
            return
        else:
            self.results = []
        for pMap in self.parameters:
            model = self.model(pMap)
            tstart = time.time()
            self.tracks = model.solve_parallel(self.events)
            runtime = time.time() - tstart 
            if self.nested:
                valMap = validate_to_json_nested(self.json_events, self.tracks, verbose=False)
            else:
                valMap = validate_to_json(self.json_events, self.tracks, verbose=False)
            valMap["inference_time"] = runtime
            valMap["parameters"] = {
                "max_slope": (pMap.get("x_slope"), pMap.get("y_slope")),
                "max_tol": (pMap.get("x_slope"), pMap.get("y_slope")),
                "scatter": pMap.get("scatter")
            }
            self.results.append(valMap)

    def optimise_parameters(self, Optimiser: optimiserBase, max_runs: int = 100) -> Dict[str, Any]:
        """ 
        Ensure the `Optimiser` is build in accordance to the OptimiserBase class 
        """ 
        i = 0
        finished = False
        self.set_pMap([Optimiser.start(algorithm=self.name)])
        with tqdm(total=max_runs, desc="Optimising") as pbar:
            while not finished:
                self.run(overwrite=True)
                Optimiser.add_run(self.get_results()[-1])
                finished = Optimiser.is_finished()
                i += 1
                pbar.update(1)
                if i >= max_runs:
                    break
                if not finished:
                    self.set_pMap([Optimiser.next_pMap()])
        if finished:
            print("Finsihed condition met, exiting...")
        return Optimiser.get_optimised_pMap()

    def set_pMap(self, pMap: Dict[str, Any]) -> None: self.parameters = pMap

    def get_results(self) -> List[Dict[str, Any]]: return self.results
    
    def calculate_db_estimate(self) -> None:
        Cv = {
            "tracks": 8.57,
            "validation": 3.64,
            "overall_db": 3.68,
            "category_db": 22.24,
            "event_db": 64.04 
        }
        Ne = len(self.events)
        Nr = len(self.parameters)
        if self.nested:
            size_bytes = Ne * Nr * (Cv.get("overall_db") + Cv.get("category_db") + Cv.get("event_db"))
        else:
            size_bytes = Ne * Nr * (Cv.get("overall_db") + Cv.get("category_db"))
        units = ["B", "KB", "MB", "GB", "TB"]
        size = size_bytes
        unit = "B"
        for u in units: # auto scale for best scale
            if size < 1024:
                unit = u
                break
            size /= 1024  
        print(f"Estimated database size: {size:.2f} {unit}")

    # note this print func is computationally heavy
    def print_validation(self, parameters: Dict = None, verbose: bool = True) -> None: 
        if not hasattr(self, "tracks"):
            if parameters == None:
                raise("Tracks not initialised, provide argument 'parameters' to compute!")
            self.tracks = self.model(parameters).solve_parallel(self.events)
        validate_print(self.json_events, self.tracks, verbose)

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
        
class TrackFollowingPipeline(PipelineBase):
    def __init__(self, events, intra_node, parameter_map=None):
        super().__init__(events, intra_node, parameter_map)
        self.name = ReconstructionAlgorithms.TRACK_FOLLOWING

    def model(self, map: Dict[str, Any]) -> TrackFollowing:
        return TrackFollowing(
            max_slopes=(map.get("x_slope"), map.get("y_slope")),
            max_tolerance=(map.get("x_tol"), map.get("y_tol")),
            max_scatter=map.get("scatter")
        )
    
class GraphDFSPipeline(PipelineBase):
    def __init__(self, events, intra_node, parameter_map=None):
        super().__init__(events, intra_node, parameter_map)
        self.name = ReconstructionAlgorithms.GRAPH_DFS
        
    def model(self, map: Dict[str, Any]) -> GraphDFS:
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
    
class SearchByTripletTriePipeline(PipelineBase):
    def __init__(self, events, intra_node, parameter_map=None):
        super().__init__(events, intra_node, parameter_map)
        self.name = ReconstructionAlgorithms.SEARCH_BY_TRIPLET_TRIE

    def model(self, map: Dict[str, Any]) -> SearchByTripletTrie:
        return SearchByTripletTrie(
            max_scatter=map.get("scatter"),
            min_strong_track_length=map.get("min_strong_track_length"),
            allowed_missed_modules=map.get("allowed_missed_modules")
        )
