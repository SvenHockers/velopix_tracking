from abc import ABC, abstractmethod
from typing import TypeAlias, cast, Union

from .algorithm_schema import ReconstructionAlgorithms
from .event_metrics import EventMetricsCalculator

pMapType: TypeAlias = dict[str, tuple[type[int]|type[float]|type[bool],None]]
ValidationResults: TypeAlias = dict[str, dict[str, list[dict[str, Union[int, float, str]]]]]

class optimiserBase(ABC):
    def __init__(self, Objective: str = "min"):
        self.objective = Objective
        if Objective == "min": self.best_score = float("inf")
        elif Objective == "max": self.best_score = float("-inf")
        else: raise(AssertionError("Specify wether the objective function should maximise or minimise the objective function!"))
        self.best_config: pMapType = {}
    
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
    
    def add_run(self, results: ValidationResults) -> None: self.run = results

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
    def objective_func(self) -> int|float: pass

    @property
    def __objective_factor(self): return -1 if self.objective == "min" else 1
    
    def event_objective(self) -> float:
        validation_results = self.get_run_data()

        n_tracks: int = cast(int, validation_results.get("total_tracks"))
        if n_tracks <= 0: return float("-inf") * self.__objective_factor

        runtime: float = cast(float, validation_results.get("inference_time"))
        ghost_rate: float = cast(float, validation_results.get("overall_ghost_rate"))
        clones: list[dict[str, float]] = cast(list[dict[str, float]], validation_results.get("categories", []))
        clone_pct: float = sum(map(lambda clone: clone.get("clone_percentage", 0), clones))

        # Note this is a really shitty optimalisation func, so please implement this for the actual algo's
        score = -(ghost_rate + clone_pct / len(clones) + runtime / 3) 
        return score * self.__objective_factor

    def intra_event_objective(self):
        calculator = EventMetricsCalculator(self.get_run_data())
        score = -calculator.get_metric(metric="clone_percentage", stat="std")

        del calculator 
        return score * self.__objective_factor + self.event_objective() 