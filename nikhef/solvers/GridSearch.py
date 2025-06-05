# pyright: strict
from .optimizers import BaseOptimizer, pMap
from collections.abc import Generator
from itertools import product
from typing import Any, TypeAlias, cast, Literal
from math import inf

import numpy as np

_Types: TypeAlias = bool | int | float
HSpace: TypeAlias = Generator[pMap, None, None] # NOTE: Stands for "hypothesis space"

class GridSearch(BaseOptimizer):
    _resolution: int
    _stopped: bool = False
    _spgen: HSpace # NOTE: Stands for "space generator"
    _options: dict[str, Any]
    _total_hypotheses: int
    _last_config: pMap
    _best_config: pMap
    _best_score: float

    def __init__(
            self, 
            resolution: int = 10, 
            objective: Literal["min", "max"] = "min", 
            nested: bool =  True,
            weights: list[float] = [1.0, 1.0, 1.0, -10.0]
            ):
        super().__init__(objective=objective, auto_eval={"autoEval": True, "nested": nested, "weights": weights})
        self._resolution = resolution
        self._options = {"w": weights, "nested": nested}

    def init(self) -> pMap:
        """
        Initializes the optimization process by setting an initial parameter map.
        """
        self._stopped = False

        num_hypotheses = 1
        axis: dict[str, tuple[_Types, ...]] = {}

        for param, (dtype, _) in self._algorithm.get_config().items():
            if dtype == bool:
                axis[param] = (False, True)
            elif dtype in (float, int):
                low, high = cast(tuple[_Types, _Types], self._algorithm.get_bounds().get(param))
                axis[param] = tuple(np.linspace(low, high, num=self._resolution, endpoint=True))
                if dtype == int:
                    axis[param] = tuple(frozenset(map(int, axis[param])))
            else:
                raise NotImplementedError(f"Unsupported type: {dtype}")
            num_hypotheses *= len(axis[param])
            
        def spgen() -> HSpace:
            idx2axe = {i: a for i, a in enumerate(axis.keys())}
            space = product(*[axis[idx2axe[i]] for i in range(len(axis))])
            space = tuple(space)
            self._total_hypotheses = len(space)
            for point in space:
                config = {idx2axe[i]: point[i] for i in range(len(point))}
                self._last_config = config
                yield config
        
        self._spgen = spgen()

        self.best_score = inf if self.objective == "min" else -inf
        try:
            self.best_config = next(self._spgen)
            return self.best_config
        except StopIteration:
            raise RuntimeError("No hypotheses generated.")

    def next(self) -> pMap:
        """
        Generates the next parameter map by slightly modifying existing values.
        """
        last_score = self.objective_func([1., 1., 1., -10.])

        if self.objective == "min":
            if last_score < self.best_score:
                self.best_score = last_score
                self.best_config = self._last_config
        elif self.objective == "max":
            if self.best_score < last_score:
                self.best_score = last_score
                self.best_config = self._last_config
        try:
            return next(self._spgen)
        except StopIteration:
            self._stopped = True
            return self.best_config

    def is_finished(self) -> bool:
        """
        Determines if the optimization process is finished.
        In this case, it stops after `max_iterations` iterations.
        """
        return self._stopped