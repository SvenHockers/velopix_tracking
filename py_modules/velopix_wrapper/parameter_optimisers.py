from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional, Union, Tuple

from .ReconstructionAlgorithms import ReconstructionAlgorithms

class optimiserBase(ABC):
    def __init__(self):
        self.best_score = float("inf")
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
    
    def add_run(self, results: Any) -> None:
        self.runs = results # might consider changing this, since this variable can get quite large in size after many runs

    def get_optimised_pMap(self) -> Dict[str, Any]:
        return self.best_config
    
    def next_pMap(self) -> Dict[str, Any]:
        """
        Return the next parameter map for the subsequent run.
        This method uses the template method pattern: it calls _generate_next_pMap (implemented by the child)
        to obtain a candidate parameter map, then validates it using self.algorithm's schema.
        """
        pMap = self.next()
        if self._algorithm is None:
            raise ValueError("Algorithm not set. Please call start() to initialise the algorithm.")
        if self.validate_config(pMap, self._algorithm.value):
            return pMap
        raise ValueError("Parameter map validation failed.")
    
    def get_run_data(self):
        return self.runs
    
    def start(self, algorithm: ReconstructionAlgorithms) -> Dict[str, Any]:
        self._algorithm = algorithm # this is required for the pMap validation
        pMap = self.init()
        if self.validate_config(pMap, self._algorithm.value):
            return pMap
        raise ValueError("Parameter map validation failed.")

    @abstractmethod
    def init(self) -> Dict[str, Any]:
        """ 
        This is the actual logic for the start call
        """
        pass

    @abstractmethod
    def next(self) -> Dict[str, Any]:
        """
        Abstract method that should generate the next parameter map.
        Child classes must implement this method. The output of this method will be automatically
        validated by the next_pMap method in the base class.
        
        :return: A dictionary with the next set of parameters.
        """
        pass

    @abstractmethod
    def score(self) -> Any:
        """ 
        Since the output of the velopix track returns a JSON array with various values (eg. ghost rates, clones, etc.)
        some method to convert these stats into a score that can be used by whatever model should be made
        """
        pass

    def is_finished(self) -> bool:
        """
        Determines if the optimisation process is finished.
        Default implementation returns True to prevent infinient loop to occur.
        """
        return True
