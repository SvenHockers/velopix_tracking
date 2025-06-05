from typing import Any, Dict, Literal, List
import random
import numpy as np
from velopix_wrappers.optimizers import BaseOptimizer, pMap

class ParticleSwarm(BaseOptimizer):
    def __init__(
        self,
        swarm_size: int = 20,
        inertia: float = 0.5,
        cognitive: float = 1.5,
        social: float = 1.5,
        max_iterations: int = 100,
        objective: Literal["min", "max"] = "min",
        nested: bool = False,
        weights: list[float] = [1.0, 1.0, 1.0, -10.0]
    ):
        super().__init__(objective=objective)
        self.swarm_size = swarm_size
        self.inertia = inertia
        self.cognitive = cognitive
        self.social = social
        self.max_iterations = max_iterations
        self.current_iteration = 0
        self.nested = nested
        self.weights = weights
        self.target_score: float = (
            float("inf") if self.objective == "max" else float("-inf")
        )

        # PSO state:
        self.swarm: List[Dict[str, Any]] = []              # Current positions
        self.velocities: List[Dict[str, float]] = []       # Velocities for each particle
        self.pbest_positions: List[Dict[str, Any]] = []    # Personal best positions
        self.pbest_scores: List[float] = []                # Personal best scores
        self.gbest_position: Dict[str, Any] = {}           # Global best position
        self.gbest_score: float = (
            float("inf") if self.objective == "min" else float("-inf")
        )

        # Bookkeeping for stepping through the swarm one‐by‐one
        self.current_particle_index: int = 0
        self._current_config: pMap = None
        self._stopped = False

    def init(self) -> pMap:
        """
        - Initialize the swarm (positions + velocities).
        - Set personal best = initial positions.
        - Find initial global best among the swarm.
        - Return the first particles configuration for evaluation.
        """
        self._stopped = False
        self.current_iteration = 0
        self.swarm.clear()
        self.velocities.clear()
        self.pbest_positions.clear()
        self.pbest_scores.clear()
        self.gbest_position = {}
        self.gbest_score = float("inf") if self.objective == "min" else float("-inf")
        self.current_particle_index = 0

        # Step 1: build swarm and velocities
        cfg = self._algorithm.get_config()
        bds = self._algorithm.get_bounds()

        for _ in range(self.swarm_size):
            pos = {}
            vel = {}

            # Randomly sample each dimension for position and velocity
            for key, (typ, _) in cfg.items():
                bounds = bds.get(key)
                if bounds is None:
                    continue
                low, high = bounds

                if typ is float:
                    # Position uniformly in [low,high]
                    pos[key] = random.uniform(low, high)
                    # Velocity uniform in [-(high-low), +(high-low)]
                    vel[key] = random.uniform(-(high - low), (high - low))
                elif typ is int:
                    pos[key] = random.randint(int(low), int(high))
                    vel[key] = random.uniform(-(high - low), (high - low))
                elif typ is bool:
                    pos[key] = random.choice([True, False])
                    vel[key] = 0.0
                elif typ is list:
                    # categorical: pick one at random; no velocity
                    pos[key] = random.choice(bounds if isinstance(bounds, list) else [])
                    vel[key] = 0.0

            self.swarm.append(pos)
            self.velocities.append(vel)

        # Step 2: initialize personal bests to the same as positions, scores unknown yet
        # We'll use +inf (for min) or -inf (for max) to force an update on first evaluation.
        initial_pbest_score = float("inf") if self.objective == "min" else float("-inf")
        for _ in range(self.swarm_size):
            self.pbest_positions.append({})
            self.pbest_scores.append(initial_pbest_score)

        # Step 3: Set the first particle to be evaluated next
        self._current_config = self.swarm[0]
        return self._current_config

    def next(self) -> pMap:
        """
        - Evaluates the current particle (self._current_config).
        - Updates personal best for that particle, updates global best if needed.
        - Advances to the next particle index. If the entire swarm has just been evaluated,
          perform a PSO velocity + position update, increment iteration count, and reset index.
        - Return the new config for evaluation.
        """
        # 1) Evaluate current particle
        score = self.objective_func(self.weights, self.nested)
        
        # Store data
        self.score_history.append(score)

        i = self.current_particle_index
        # Update personal best if this is better
        if self.pbest_positions[i] is None or self.pbest_positions[i] == {}:
            # First‐time initialization of pbest
            self.pbest_positions[i] = self._current_config.copy()
            self.pbest_scores[i] = score
        else:
            if self.objective == "min":
                if score < self.pbest_scores[i]:
                    self.pbest_scores[i] = score
                    self.pbest_positions[i] = self._current_config.copy()
            else:  # "max"
                if score > self.pbest_scores[i]:
                    self.pbest_scores[i] = score
                    self.pbest_positions[i] = self._current_config.copy()

        # Update global best if needed
        if self.objective == "min":
            if score < self.gbest_score:
                self.gbest_score = score
                self.gbest_position = self._current_config.copy()
        else:
            if score > self.gbest_score:
                self.gbest_score = score
                self.gbest_position = self._current_config.copy()

        # 2) Advance to next particle (or, if end of swarm, perform a PSO update)
        self.current_particle_index += 1

        if self.current_particle_index >= self.swarm_size:
            # We have finished evaluating all particles in this iteration
            # Check stopping criteria before updating
            if (
                (self.objective == "min" and self.gbest_score <= self.target_score)
                or (self.objective == "max" and self.gbest_score >= self.target_score)
            ):
                self._stopped = True

            # If we still have iterations left, update velocities & positions
            if self.current_iteration < self.max_iterations and not self._stopped:
                self._update_velocities_positions()
                self.current_iteration += 1
                self.current_particle_index = 0
            else:
                self._stopped = True

        # 3) Prepare the next config to evaluate
        if not self._stopped:
            self._current_config = self.swarm[self.current_particle_index]
        return self._current_config

    def _update_velocities_positions(self):
        """
        After a full pass of evaluations, update each particle's velocity & position:
        v <- w*v + c1*r1*(pbest - pos) + c2*r2*(gbest - pos)
        pos ← pos + v
        For non numeric types (bool, list), we simply resample randomly each update.
        """
        cfg = self._algorithm.get_config()
        bds = self._algorithm.get_bounds()

        for i in range(self.swarm_size):
            pos = self.swarm[i]
            vel = self.velocities[i]
            pbest = self.pbest_positions[i]
            # Fallback if pbest is missing (should only occur before first evaluation)
            if not pbest:
                pbest = pos

            for key, (typ, _) in cfg.items():
                bounds = bds.get(key)
                if bounds is None:
                    continue
                low, high = bounds

                if typ is float:
                    # Retrieve current float position & velocity
                    x = float(pos[key])
                    v = float(vel.get(key, 0.0))
                    pbest_x = float(pbest[key])
                    gbest_x = float(self.gbest_position[key])

                    r1 = random.random()
                    r2 = random.random()

                    # PSO velocity update for float
                    new_v = (
                        self.inertia * v
                        + self.cognitive * r1 * (pbest_x - x)
                        + self.social * r2 * (gbest_x - x)
                    )

                    # Update position and clamp
                    x_new = x + new_v
                    x_new = max(min(x_new, high), low)

                    # Write back as float
                    pos[key] = float(x_new)
                    vel[key] = new_v

                elif typ is int:
                    # Retrieve current int position as float & velocity
                    x = float(pos[key])
                    v = float(vel.get(key, 0.0))
                    pbest_x = float(pbest[key])
                    gbest_x = float(self.gbest_position[key])

                    r1 = random.random()
                    r2 = random.random()

                    # PSO velocity update for int
                    new_v = (
                        self.inertia * v
                        + self.cognitive * r1 * (pbest_x - x)
                        + self.social * r2 * (gbest_x - x)
                    )

                    # Update position and clamp in float space
                    x_new = x + new_v
                    x_new = max(min(x_new, high), low)

                    # Round & clamp to integer
                    x_new_int = int(round(x_new))
                    x_new_int = max(min(x_new_int, int(high)), int(low))

                    pos[key] = x_new_int
                    vel[key] = new_v

                elif typ is bool:
                    # No velocity for boolean; resample randomly each iteration
                    pos[key] = random.choice([True, False])
                    vel[key] = 0.0

                elif typ is list:
                    # Categorical: pick a random choice each iteration
                    options = bounds if isinstance(bounds, list) else []
                    if options:
                        pos[key] = random.choice(options)
                    vel[key] = 0.0

            # Save updates back into swarm and velocities lists
            self.swarm[i] = pos
            self.velocities[i] = vel

    def is_finished(self) -> bool:
        return self._stopped
