from abc import ABC, abstractmethod
import time
# import pandas as pd
from .validation_to_datasets import *
from velopix import Event, Track, TrackFollowing, GraphDFS, SearchByTripletTrie, \
                            validate_print, validate_to_json_nested, validate_to_json
from .optimizers import BaseOptimizer
from .algorithm_schema import ReconstructionAlgorithms
from .custom_types import *
from typing import cast#, Optional
# from tqdm import tqdm
import warnings

class PipelineBase(ABC):
    def __init__(self, events: EventType, intra_node: bool, parameter_map: list[pMap]|None = None) -> None:
        self.name: ReconstructionAlgorithms
        self.json_events = events
        self.nested = intra_node
        self.parameters = parameter_map
        self.events: list[Event] = []
        for event in events:
            self.events.append(Event(event))

    @abstractmethod
    def model(self, pmap: pMap) -> ReconstructionAlgorithmsType: pass

    def run(self, overwrite: bool) -> None:
        # here I should include a warning if self.results != empty break to prevent loss of data
        if hasattr(self, "results") and not overwrite:
            warnings.warn("Overwriting results. This will cause a loss of data!", UserWarning)
            return
        else:
            self.results: list[ValidationResults] = []
        for pmap in cast(list[pMap], self.parameters):
            model: ReconstructionAlgorithmsType = self.model(pmap)
            tstart = time.time()
            self.tracks: list[Track] = model.solve_parallel(self.events)
            runtime = time.time() - tstart 
            if self.nested:
                valMap = validate_to_json_nested(self.json_events, self.tracks, verbose=True) # type: ignore
            else:
                valMap = validate_to_json(self.json_events, self.tracks, verbose=False) # type: ignore
            valMap["inference_time"] = runtime
            valMap["parameters"] = {
                "max_slope": (pmap.get("x_slope"), pmap.get("y_slope")), 
                "max_tol": (pmap.get("x_slope"), pmap.get("y_slope")), 
                "scatter": pmap.get("scatter") 
            }
            self.results.append(valMap)

    def optimise_parameters(self, Optimiser: BaseOptimizer, max_runs: int = 100) -> pMap:
        """ 
        Ensure the `Optimiser` is build in accordance to the OptimiserBase class 
        """ 
        i = 0
        finished = False
        self.set_pMap([Optimiser.start(algorithm=self.name)])
        # with tqdm(total=max_runs, desc="Optimising") as pbar:
        while not finished:
            self.run(overwrite=True)
            Optimiser.add_run(self.get_results()[-1])
            finished = Optimiser.is_finished()
            i += 1
            # pbar.update(1)
            if i >= max_runs:
                break
            if not finished:
                self.set_pMap([Optimiser.next_pMap()])
        # if finished:
            # print("Finsihed condition met, exiting...")
        return Optimiser.get_optimised_pMap()

    def set_pMap(self, pmap: list[pMap]) -> None: self.parameters = pmap

    def get_results(self) -> list[dict[str, Any]]: return self.results
    
    def calculate_db_estimate(self) -> None:
        Cv = {
            "tracks": 8.57,
            "validation": 3.64,
            "overall_db": 3.68,
            "category_db": 22.24,
            "event_db": 64.04 
        }
        Ne = len(self.events)
        Nr = len(self.parameters or [])
        if self.nested:
            size_bytes: float = Ne * Nr * (Cv.get("overall_db") + Cv.get("category_db") + Cv.get("event_db")) # type: ignore
        else:
            size_bytes: float = Ne * Nr * (Cv.get("overall_db") + Cv.get("category_db")) # type: ignore
        units = ["B", "KB", "MB", "GB", "TB"]
        size: float = cast(float, size_bytes)
        unit = "B"
        for u in units: # auto scale for best scale
            if size < 1024:
                unit = u
                break
            size /= 1024  
        print(f"Estimated database size: {size:.2f} {unit}")

    # note this print func is computationally heavy
    def print_validation(self, parameters: pMap|None = None, verbose: bool = True) -> None: 
        if not hasattr(self, "tracks"):
            if parameters == None:
                raise(AssertionError)
            self.tracks: list[Track] = self.model(parameters).solve_parallel(self.events) 
        validate_print(self.json_events, self.tracks, verbose) # type: ignore

    # def generate_database(self, output_directory: str, overwrite: bool) -> None:
    #     func = "output_distributions" if self.nested else "output_aggregates"
    #     save_to_file(results=self.results, directory=output_directory, output_func=func, overwrite=overwrite)

    # def generate_and_get_database(self) -> tuple[pd.DataFrame, pd.DataFrame, Optional[pd.DataFrame]]:
    #     sfunc = "output_distributions" if self.nested else "output_aggregates"
    #     func: Callable[[list[ValidationResults]], Any] = getattr(sfunc, None) # type: ignore
    #     if not callable(func):
    #         raise ValueError(f"Output function '{sfunc}' not found.")
        
    #     if sfunc == "output_aggregates":
    #         return func(self.results)
    #     else:
    #         return func(self.results)
        
class TrackFollowingPipeline(PipelineBase):
    def __init__(self, events: EventType, intra_node: bool, parameter_map: list[pMap]|None=None):
        super().__init__(events, intra_node, parameter_map)
        self.name = ReconstructionAlgorithms.TRACK_FOLLOWING

    def model(self, pmap: pMap) -> TrackFollowing:
        return TrackFollowing(
            max_slopes=(pmap.get("x_slope"), pmap.get("y_slope")), # type: ignore
            max_tolerance=(pmap.get("x_tol"), pmap.get("y_tol")), # type: ignore
            max_scatter=pmap.get("scatter") # type: ignore
        )
    
class GraphDFSPipeline(PipelineBase):
    def __init__(self, events: EventType, intra_node: bool, parameter_map: list[pMap]|None=None):
        super().__init__(events, intra_node, parameter_map)
        self.name = ReconstructionAlgorithms.GRAPH_DFS
        
    def model(self, pmap: pMap) -> GraphDFS:
        return GraphDFS(
            max_slopes=(pmap.get("x_slope"), pmap.get("y_slope")), # type: ignore
            max_tolerance=(pmap.get("x_tol"), pmap.get("y_tol")), # type: ignore
            max_scatter=pmap.get("scatter"), # type: ignore
            minimum_root_weight=pmap.get("minimum_root_weight"), # type: ignore
            weight_assignment_iterations=pmap.get("weight_assignment_iterations"), # type: ignore
            allowed_skip_modules=pmap.get("allowed_skip_modules"), # type: ignore
            allow_cross_track=pmap.get("allow_cross_track"), # type: ignore
            clone_ghost_killing=pmap.get("clone_ghost_killing") # type: ignore
        )
    
class SearchByTripletTriePipeline(PipelineBase):
    def __init__(self, events: EventType, intra_node: bool, parameter_map: list[pMap]|None=None):
        super().__init__(events, intra_node, parameter_map)
        self.name = ReconstructionAlgorithms.SEARCH_BY_TRIPLET_TRIE

    def model(self, pmap: pMap) -> SearchByTripletTrie:
        return SearchByTripletTrie(
            max_scatter=pmap.get("scatter"), # type: ignore
            min_strong_track_length=pmap.get("min_strong_track_length"), # type: ignore
            allowed_missed_modules=pmap.get("allowed_missed_modules") # type: ignore
        )