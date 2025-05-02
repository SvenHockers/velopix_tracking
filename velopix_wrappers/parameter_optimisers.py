from abc import ABC, abstractmethod
from copy import deepcopy
from typing import cast

from .algorithm_schema import ReconstructionAlgorithms
from .event_metrics import EventMetricsCalculator
from .custom_types import *

class optimiserBase(ABC):
    def __init__(self, Objective: str = "min", auto_eval: dict[str, bool|list[float]] = {"autoEval": False, "nested": False, "weights": [0,0,0]}):
        self.objective = Objective
        if Objective == "min": self.best_score = float("inf")
        elif Objective == "max": self.best_score = float("-inf")
        else: raise(AssertionError("Specify wether the objective function should maximise or minimise the objective function!"))
        self.best_config: pMapType = {}
        self.auto_evaluate: bool = cast(bool, auto_eval.get("autoEval"))
        if self.auto_evaluate:
            self.nested = cast(bool, auto_eval.get("nested"))
            self.weights = cast(list[float], auto_eval.get("weights"))
            self.score_history: list[float] = []
            self.history: dict[str, float] = {}
            self.prev_config: pMapType = {}
    
    @staticmethod
    def validate_config(config: pMapType, schema: pMapType) -> bool:
        """
        Validates that the keys in the parameter map match the expected types defined.
        """
        for key, (expected_type, _) in schema.items():
            if key in config and not isinstance(config[key], expected_type):
                raise TypeError(f"Expected type {expected_type} for key '{key}', got {type(config[key])}.")
        return True
    
    def start(self, algorithm: ReconstructionAlgorithms) -> pMapType:
        self._algorithm = algorithm # this is required for the pMap validation
        pMap = self.init()
        if self.validate_config(pMap, self._algorithm.value):
            return pMap
        raise ValueError("Parameter map validation failed.")

    def next_pMap(self) -> pMapType:
        """
        Return the next parameter map to the pipeline, this invokes next() method where the actual
        logic is. And does an additional schema validation on it.
        """
        pMap = self.next()
        if self.validate_config(pMap, self._algorithm.value):
            return pMap
        raise ValueError("Parameter map validation failed.")
    
    def add_run(self, results: ValidationResults) -> None: 
        if self.auto_evaluate:
            self._evaluate_run(weight=self.weights, nested=self.nested)
        self.run = results

    def get_optimised_pMap(self) -> pMapType: return self.best_config

    def get_run_data(self) -> ValidationResults: return self.run
    
    """  
    Algorithm implementation methods
        init -> initialise the algorithm
        next -> next itteration
        is_finsihed -> define convergence conditions :: True == Done, False != Done
    """

    @abstractmethod
    def init(self) -> pMapType: pass

    @abstractmethod
    def next(self) -> pMapType: pass

    @abstractmethod
    def is_finished(self) -> bool: return True
    
    """ 
    Objective Function methods:
    """

    @abstractmethod
    def objective_func(self, w: list[float], nested: bool = False) -> int|float: 
        if nested:
            return self.intra_event_objective(w)
        return self.event_objective(w)

    @property
    def _objective_factor(self): return -1 if self.objective == "min" else 1
    
    def event_objective(self, weights: list[float]) -> float:
        validation_results = self.get_run_data()

        n_tracks: int = cast(int, validation_results.get("total_tracks"))
        if n_tracks <= 0: return float("-inf") * self._objective_factor

        runtime: float = cast(float, validation_results.get("inference_time"))
        ghost_rate: float = cast(float, validation_results.get("overall_ghost_rate"))
        clones: list[dict[str, float]] = cast(list[dict[str, float]], validation_results.get("categories", []))
        clone_pct: float = sum(map(lambda clone: clone.get("clone_percentage", 0), clones))

        # Note this is a really shitty optimalisation func, so please implement this for the actual algo's
        score = -(weights[0] * ghost_rate + weights[1] * clone_pct / len(clones) + weights[2] * runtime / 3) 
        return score * self._objective_factor

    def intra_event_objective(self, weights: list[float]) -> float:
        calculator = EventMetricsCalculator(self.get_run_data())
        score = -calculator.get_metric(metric="clone_percentage", stat="std")

        del calculator 
        return score * self._objective_factor + self.event_objective(weights) 
    
    def _evaluate_run(self, weight: list[float], nested: bool = False) -> None:
        score = self.objective_func(weight, nested)
        if score == None: print("Score is null") # type: ignore
        self.score_history.append(score)
        self.history[str(self.prev_config.items())] = deepcopy(score) # I changed tuple(self.prev_config.items()) -> str(self.prev_config.items()) not sure if this works
        compare = score < self.best_score if self.objective == "min" else score > self.best_score
        if compare:
            self.best_score = score
            self.best_config = deepcopy(self.prev_config)