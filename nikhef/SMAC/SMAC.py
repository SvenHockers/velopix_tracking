import os, sys, json
import numpy as np 
from typing import cast
from copy import deepcopy
from sklearn.ensemble import RandomForestRegressor
from scipy.stats import norm
import random
import json

from dependensies.parameter_optimisers import optimiserBase
from dependensies.velopix_pipeline import TrackFollowingPipeline
from dependensies.event_metrics import EventMetricsCalculator
from dependensies.custom_types import *

class SMAC(optimiserBase):
    def __init__(self, numberOfContextDataPoints: int, MinimalNumberOfRuns: int, numberOfCandidates: int, objectiveWeights: list[float], randomForestEstimators: int, intra_event: bool = False) -> None:
        super().__init__(Objective="min")
        """
        history structure I'll use here:
        Results : {
            parameters: {...},
            score: float
        }
        """
        self.history = {} 
        self.intra_event = intra_event
        self.w = objectiveWeights
        self.score_history = []
        self.patience = MinimalNumberOfRuns
        self.convergence_tolerance = 1e-4
        self.context_iterations = numberOfContextDataPoints
        self.RFestimators = randomForestEstimators
        self.n_candidates = numberOfCandidates

    # This init method randomly initialises the parameteres and could be used in other implementations
    def init(self) -> pMapType:
        self.iteration = 0
        self.prev_config: MetricsDict = self.__random_sample()
        return deepcopy(cast(pMapType, self.prev_config))

    def next(self) -> pMapType:
        self.__evaluate_run()

        self.iteration += 1

        # SMAC uses random configurations initialy to build a RF model
        if self.iteration <= self.context_iterations:
            self.prev_config = self.__random_sample()
            return deepcopy(cast(pMapType, self.prev_config))
        
        # Build the RF using the context gained
        X = np.array([self.__to_vec(c) for c in self.history])
        y = np.array([s for s in self.history.values()])

        # NOTE this is fairly expensive, retraining a RF everytime we update the model
        RF = RandomForestRegressor(n_estimators=self.RFestimators)
        RF.fit(X, y) # type: ignore

        # here the actual update func starts
        best_ei = -float("inf")
        best_candidate = None
        for _ in range(self.n_candidates):
            candidate = self.__random_sample()
            x_vec = np.array(self.__to_vec(candidate)).reshape(1, -1)
            preds = np.array([est.predict(x_vec)[0] for est in RF.estimators_]) # type: ignore
            mu = np.mean(preds)
            sigma = np.std(preds)
            # here calculate expected improvement
            if sigma > 0:
                z = (self.best_score - mu) / sigma
                expectedImprov = (self.best_score - mu) * norm.cdf(z) + sigma * norm.pdf(z) # type: ignore
            else:
                expectedImprov = 0.0
            if expectedImprov > best_ei:
                best_ei = expectedImprov
                best_candidate = candidate

        # if no candidate is selected return random config
        if best_candidate is None:
            best_candidate = self.__random_sample()

        self.prev_config = best_candidate
        return deepcopy(cast(pMapType, self.prev_config))


    def is_finished(self) -> bool:
        if len(self.score_history) > self.patience:
            recent_scores = self.score_history[-self.patience:]
            improvement = max(recent_scores) - min(recent_scores)
            if improvement < self.convergence_tolerance:
                return True
        return False
    
    def objective_func(self, w: list[float] = [], nested: bool = False) -> float:
        results = self.get_run_data()

        n_tracks: int = cast(int, results.get("total_tracks"))
        if n_tracks <= 0: return float("-inf") * self._objective_factor

        runtime = cast(float, results.get("inference_time"))
        ghost_rate = cast(float, results.get("overall_ghost_rate"))
        clones = cast(list[dict[str, Any]], results.get("categories", []))
        if clones:
            clone_pct = sum(map(lambda clone: clone.get("clone_percentage", 0), clones)) / len(clones)
        else:
            clone_pct = 100

        if self.intra_event:
            calculator = EventMetricsCalculator(self.get_run_data())
            # intra_hit_eff = -calculator.get_metric(metric="avg_purity", stat="std")
            intra_clone_rate = calculator.get_metric(metric="clone_percentage", stat="std")
            return self.w[0] * ghost_rate +  self.w[1] * clone_pct + self.w[2] * runtime + self.w[4] * intra_clone_rate
        return self.w[0] * ghost_rate +  self.w[1] * clone_pct + self.w[2] * runtime 
    """ 
    Internal Helper Classes 
    """

    def __random_sample(self) -> MetricsDict:
        schema = self._algorithm.get_config() # Get the algorithm parameters schema
        config: MetricsDict = {}

        for key, (type, _) in schema.items(): # Iterate over the parameters and intialise randomly them according to there type
            if type == bool:
                config[key] = random.choice([True, False])
            elif type in (float, int):
                low, high = cast(tuple[int|float, int|float], self._algorithm.get_bounds().get(key))
                config[key] = type(random.uniform(low, high))
        return config
    
    def __evaluate_run(self) -> None:
        score = self.objective_func()
        if score == None: pass
        self.score_history.append(score)
        self.history[tuple(self.prev_config.items())] = deepcopy(score)
        compare = score < self.best_score if self.objective == "min" else score > self.best_score
        if compare:
            self.best_score = score
            self.best_config = deepcopy(cast(pMapType, self.prev_config))

    def __to_vec(self, config: MetricsDict) -> list[float]:
        vec: list[float] = []
        if isinstance(config, tuple):
            config = dict(config)
        for key in list(self._algorithm.get_config().keys()):
            value = config[key]
            if isinstance(value, bool):
                vec.append(1.0 if value else 0.0)
            else:
                vec.append(float(value))
        return vec

if __name__ == "__main__":
    events = []
    n_files = 100

    for i in range(0, n_files):
        if i == 51:
            """
            There's an issue with event 51 -> module_prefix_sum contains value 79 twice resulting in and indexing error when loading the event
            """
            pass
        else:
            event_file = open(os.path.join("../DB/raw", f"velo_event_{i}.json"))
            json_data = json.loads(event_file.read())
            events.append(json_data)
            event_file.close()

    OptimiseWithinBatches = False
    optimiser = SMAC(
        numberOfContextDataPoints=25, 
        MinimalNumberOfRuns=60, 
        numberOfCandidates = 5,
        objectiveWeights = [5, 1, 0.2, 0.1, 0.5],
        randomForestEstimators=15, # For each parameter 3
        intra_event=False
    )

    optimalParameters = TrackFollowingPipeline(events=events, intra_node=OptimiseWithinBatches).optimise_parameters(Optimiser, max_runs=15000)

    with open("SMAC_TrackFollowing.json", "w") as out_file:
        json.dump(optimiser.history, out_file, indent=4)