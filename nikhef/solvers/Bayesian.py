from typing import Any, Literal, List
import random
import numpy as np
from sklearn.gaussian_process import GaussianProcessRegressor
from sklearn.gaussian_process.kernels import RBF, ConstantKernel as C
from .optimizers import BaseOptimizer, pMap 

"""  
ChangeLog:

1) Check imports -> velopix_wrappers have been mv to dependensies
2) Note I pass arguments now via **kwargs, this works if the strings EXACTLY matches otherwise you will run into errors
"""

class Bayesian(BaseOptimizer):
    def __init__(
        self,
        learning_rate: float,
        max_iterations: int = 100,
        target_score: float = 0.3,
        objective: Literal["min", "max"] = "min",
        nested: bool = True,
        weights: list[float] = [1.0, 1.0, 1.0, -10.0]
    ): 
        super().__init__(objective=objective, auto_eval={"autoEval": True, "nested": nested, "weights": weights})
        self.learning_rate = learning_rate
        self.max_iterations = max_iterations
        self.target_score = target_score
        self.current_iteration = 0
        self.nested = nested
        self.weights = weights

        # Storage for GP model
        self.X: List[List[Any]] = []
        self.Y: List[float] = []

        # Gaussian Process model
        kernel = C(1.0, (1e-4, 1e1)) * RBF(1.0, (1e-4, 1e1))
        self.gpr = GaussianProcessRegressor(kernel=kernel, n_restarts_optimizer=10)
        
        # State tracking
        self._stopped = False
        self._current_config = None

    def init(self) -> pMap:
        """
        Initializes with a random point within bounds.
        """
        self._stopped = False
        self.current_iteration = 0
        self.X = []
        self.Y = []
        
        # Sample initial point
        initial_config = self._sample_random_point()
        
        # Store for evaluation in first call to next()
        self._current_config = initial_config
        
        return initial_config

    def next(self) -> pMap:
        """
        Evaluates the current configuration and returns a new one.
        """
        # Evaluate the current configuration (from previous init/next call)
        score = self.objective_func(self.weights, self.nested)
        
        # Record evaluation
        if self._current_config:
            self.X.append(list(self._current_config.values()))
            self.Y.append(score)
            
            # Update best if better
            if self.objective == "min":
                if score < self.best_score:
                    self.best_score = score
                    self.best_config = self._current_config.copy()
            else:  # "max"
                if score > self.best_score:
                    self.best_score = score
                    self.best_config = self._current_config.copy()
        
        # Generate next point
        if len(self.X) > 1:  # Need at least 2 points to fit GP
            self.gpr.fit(np.array(self.X), np.array(self.Y))
            next_config = self._predict_next()
        else:
            next_config = self._sample_random_point()
        
        # Store for evaluation in next call
        self._current_config = next_config
        self.current_iteration += 1
        
        # Check if we've reached the iteration limit
        if self.current_iteration >= self.max_iterations:
            self._stopped = True
            
        return next_config

    def _predict_next(self) -> pMap:
        """
        Uses GP to predict promising points.
        """
        cfg = self._algorithm.get_config()
        bds = self._algorithm.get_bounds()
        
        # Sample a set of random candidates
        num_candidates = 20
        candidates = []
        
        for _ in range(num_candidates):
            candidate = {}
            for key, (typ, _) in cfg.items():
                bounds = bds.get(key)
                if bounds is None:
                    continue
                    
                low, high = bounds
                if typ is float:
                    candidate[key] = random.uniform(low, high)
                elif typ is int:
                    candidate[key] = random.randint(int(low), int(high))
                elif typ is bool:
                    candidate[key] = random.choice([True, False])
                elif typ is list:
                    if isinstance(bounds, list):
                        candidate[key] = random.choice(bounds)
                    else:
                        candidate[key] = []
            candidates.append((list(candidate.values()), candidate))
            
        # Evaluate candidates with acquisition function
        best_val = float("inf") if self.objective == "min" else float("-inf")
        best_candidate = candidates[0][1] if candidates else self._sample_random_point()
        
        for values, cand_dict in candidates:
            x_candidate = np.array(values).reshape(1, -1)
            mean, std = self.gpr.predict(x_candidate, return_std=True)
            
            # Acquisition function (Expected Improvement)
            if self.objective == "min":
                acq_value = float(mean) - 0.1 * float(std)
                if acq_value < best_val:
                    best_val = acq_value
                    best_candidate = cand_dict
            else:  # maximization
                acq_value = float(mean) + 0.1 * float(std)
                if acq_value > best_val:
                    best_val = acq_value
                    best_candidate = cand_dict
                
        return best_candidate

    def _sample_random_point(self) -> pMap:
        """Helper to generate a random point within bounds."""
        cfg = self._algorithm.get_config()
        bds = self._algorithm.get_bounds()
        params = {}
        
        for key, (typ, _) in cfg.items():
            bounds = bds.get(key)
            if bounds is None:
                # NOTE: What do we do if the bound is not specified? Please realize that this will cause a silent failure that's hard to debug.
                # - by Dennis
                continue
                
            low, high = bounds
            if typ is float:
                params[key] = random.uniform(low, high)
            elif typ is int:
                params[key] = random.randint(int(low), int(high))
            elif typ is bool:
                params[key] = random.choice([True, False])
            elif typ is list:
                if isinstance(bounds, list):
                    params[key] = random.choice(bounds)
                else:
                    params[key] = []
        return params

    def is_finished(self) -> bool:
        """Determines if optimization is complete."""
        if self._stopped:
            return True