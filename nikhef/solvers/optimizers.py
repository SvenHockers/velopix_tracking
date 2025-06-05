from abc import ABC, abstractmethod
from collections.abc import Sequence
from copy import deepcopy
from typing import cast
from uuid import UUID, uuid4

from .algorithm_schema import ReconstructionAlgorithms
from .event_metrics import EventMetricsCalculator
from math import nan
from .custom_types import *


class BaseOptimizer(ABC):
    def __init__(self, objective: str = "min", auto_eval: dict[str, bool|list[float]] = {"autoEval": False, "nested": True, "weights": []}):
        self.objective = objective
        if objective == "min": self.best_score = float("inf")
        elif objective == "max": self.best_score = float("-inf")
        else: raise(AssertionError("Specify wether the objective function should maximise or minimise the objective function!"))
        self.best_config: pMap = {}
        self.auto_evaluate: bool = cast(bool, auto_eval.get("autoEval"))
        if self.auto_evaluate:
            self.nested = cast(bool, auto_eval.get("nested"))
            self.weights = cast(list[float], auto_eval.get("weights"))
            self.score_history: list[float] = []
            self.history: dict[str, Any] = {}
            self.prev_config: pMap = {}
    
    @staticmethod
    def validate_config(config: pMap, schema: pMapType) -> bool:
        """
        Validates that the keys in the parameter map match the expected types defined.
        """
        for key, (expected_type, _) in schema.items():
            if key in config and not isinstance(config[key], expected_type):
                raise TypeError(f"Expected type {expected_type} for key '{key}', got {type(config[key])}.")
        return True
    
    def start(self, algorithm: ReconstructionAlgorithms) -> pMap:
        self._algorithm = algorithm # this is required for the pMap validation
        pmap = self.init()
        if self.validate_config(pmap, self._algorithm.value):
            return pmap
        raise ValueError("Parameter map validation failed.")

    def next_pMap(self) -> pMap:
        """
        Return the next parameter map to the pipeline, this invokes next() method where the actual
        logic is. And does an additional schema validation on it.
        """
        pmap = self.next()
        if self.validate_config(pmap, self._algorithm.value):
            self.prev_config = pmap
            return pmap
        raise ValueError("Parameter map validation failed.")
    
    def add_run(self, results: ValidationResults) -> None: 
        self.run = results
        if self.auto_evaluate:
            self._evaluate_run(validationResult=results, weight=self.weights, nested=self.nested)
        

    def get_optimised_pMap(self) -> pMap: return self.best_config

    def get_run_data(self) -> ValidationResults: return self.run
    
    """  
    Algorithm implementation methods
        init -> initialise the algorithm
        next -> next itteration
        is_finsihed -> define convergence conditions :: True == Done, False != Done
    """

    @abstractmethod
    def init(self) -> pMap: pass

    @abstractmethod
    def next(self) -> pMap: pass

    @abstractmethod
    def is_finished(self) -> bool: return True
    
    """ 
    Objective Function methods:
    """

    def objective_func(self, weights: Sequence[float], nested: bool = True) -> int|float:
        run_data = self.get_run_data()
        time_rate = cast(float, run_data.get("inference_time", nan))
        ghost_rate = cast(float, run_data.get("overall_ghost_rate", nan))
        num_tracks = cast(float, run_data.get("total_tracks", nan))
        penalty = 0
        if num_tracks == 0:
            penalty = 999_999 if self.objective == "min" else -999_999
        if nested:
            calculator = EventMetricsCalculator(run_data)
            clone_rate = calculator.get_metric(metric="clone_percentage", stat="mean")
            terms = (time_rate, clone_rate, ghost_rate, num_tracks)
            return sum(w * t for w, t in zip(weights, terms, strict=True)) + penalty
        terms = (time_rate, ghost_rate, num_tracks)
        return sum(w * t for w, t in zip(weights, terms, strict=True)) + penalty
    
    def _evaluate_run(self, validationResult: ValidationResults | ValidationResultsNested, weight: list[float], nested: bool = False) -> None:
        score = self.objective_func(weight, nested)
        self.history[str(uuid4())] = {
            "params": deepcopy(self.prev_config),
            "score": deepcopy(score),
            "meta": deepcopy(validationResult)
        }
        if score is None: # type:ignore
            return
        self.score_history.append(score)
        compare = score < self.best_score if self.objective == "min" else score > self.best_score
        if compare:
            self.best_score = score
            self.best_config = deepcopy(self.prev_config)