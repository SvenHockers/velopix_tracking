from abc import ABC, abstractmethod
from typing import Any, Dict
import pandas as pd

from .algorithm_schema import ReconstructionAlgorithms
from .event_metrics import EventMetricsCalculator

class optimiserBase(ABC):
    def __init__(self, Objective: str = "min"):
        self.objective = Objective
        if Objective == "min": self.best_score = float("inf")
        elif Objective == "max": self.best_score = float("-inf")
        else: raise(AssertionError("Specify wether the objective function should maximise or minimise the objective function!"))
        self.best_config = {}
    
    @staticmethod
    def validate_config(config: dict, schema: dict) -> bool:
        """
        Validates that the keys in the parameter map match the expected types defined.
        """
        for key, (expected_type, _) in schema.items():
            if key in config and not isinstance(config[key], expected_type):
                raise TypeError(f"Expected type {expected_type} for key '{key}', got {type(config[key])}.")
        return True
    
    def start(self, algorithm: ReconstructionAlgorithms) -> Dict[str, Any]:
        self._algorithm = algorithm # this is required for the pMap validation
        pMap = self.init()
        if self.validate_config(pMap, self._algorithm.value):
            return pMap
        raise ValueError("Parameter map validation failed.")

    def next_pMap(self) -> Dict[str, Any]:
        """
        Return the next parameter map to the pipeline, this invokes next() method where the actual
        logic is. And does an additional schema validation on it.
        """
        pMap = self.next()
        if self._algorithm is None:
            raise ValueError("Algorithm not set. Please call start() to initialise the algorithm.")
        if self.validate_config(pMap, self._algorithm.value):
            return pMap
        raise ValueError("Parameter map validation failed.")
    
    def add_run(self, results: Any) -> None: self.run = results

    def get_optimised_pMap(self) -> Dict[str, Any]: return self.best_config

    def get_run_data(self) -> Dict[str, Any]: return self.run
    
    """  
    Algorithm implementation methods
        init -> initialise the algorithm
        next -> next itteration
        is_finsihed -> define convergence conditions :: True == Done, False != Done
    """

    @abstractmethod
    def init(self) -> Dict[str, Any]: pass

    @abstractmethod
    def next(self) -> Dict[str, Any]: pass

    @abstractmethod
    def is_finished(self) -> bool: return True
    
    """ 
    Objective Function methods:
    """

    @abstractmethod
    def objective_func(self) -> Any: pass

    @property
    def __objective_factor(self): return -1 if self.objective == "min" else 1
    
    def event_objective(self):
        validation_results = self.get_run_data()

        n_tracks = validation_results.get("total_tracks")
        if n_tracks <= 0: return float("-inf") * self.__objective_factor

        runtime = validation_results.get("inference_time") 
        ghost_rate = validation_results.get("overall_ghost_rate")
        clones = validation_results.get("categories", []) 
        clone_pct = sum(map(lambda clone: clone.get("clone_percentage", 0), clones))

        score = -(ghost_rate + clone_pct / len(clones) + runtime / 3) 
        return score * self.__objective_factor

    def intra_event_objective(self):
        calculator = EventMetricsCalculator(self.get_run_data())
        score = -calculator.get_metric(metric="clone_percentage", stat="std")

        del calculator 
        return score * self.__objective_factor + self.event_objective() 