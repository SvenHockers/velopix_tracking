from typing import Any, List, Optional, Tuple, Dict

"""
Virtual Env
"""

class Hit:
    id: int
    x: float
    y: float
    z: float
    t: float
    module_number: int
    with_t: bool

    def __init__(
        self, 
        x: float, 
        y: float, 
        z: float, 
        hit_id: int, 
        module: Optional[int] = ..., 
        t: Optional[float] = ..., 
        with_t: Optional[bool] = ...
    ) -> None: ...
    
    def __repr__(self) -> str: ...
    
    def __eq__(self, other: object) -> bool: ...
    
    def __hash__(self) -> int: ...

class Track:
    hits: List[Hit]
    missed_last_module: bool
    missed_penultimate_module: bool

    def __init__(self, hits: List[Hit]) -> None: ...
    
    def add_hit(self, hit: Hit) -> None: ...
    
    def __repr__(self) -> str: ...

class Module:
    module_number: int
    z: float
    hit_start_index: int
    hit_end_index: int
    global_hits: List[Hit]

    def __init__(self, module_number: int, z: float, hit_start_index: int, hit_end_index: int, global_hits: List[Hit]) -> None: ...
    
    def hits(self) -> List[Hit]: ...
    
    def __repr__(self) -> str: ...


class Event:
    description: str
    montecarlo: object  
    module_prefix_sum: List[int]
    number_of_hits: int
    module_zs: List[List[float]]
    hits: List[Hit]
    modules: List[Module]

    def __init__(self, json_data: Any) -> None: ...


"""
Algorithms
"""

class TrackFollowing:
    def __init__(
        self, 
        max_slopes: Optional[Tuple[float, float]] = None,
        max_tolerance: Optional[Tuple[float, float]] = None,
        max_scatter: Optional[float] = None,
        min_track_length: Optional[int] = None,
        min_strong_track_length: Optional[int] = None
    ) -> None: ...
    
    def are_compatible(self, hit0: Hit, hit1: Hit) -> bool: ...
    
    def check_tolerance(self, hit0: Hit, hit1: Hit, hit2: Hit) -> bool: ...
    
    def solve(self, event: Event) -> List[Track]: ...
    
    def solve_parallel(self, events: List[Event]) -> List[Track]: ...

class GraphDFS:
    def __init__(
        self,
        max_slopes: Optional[Tuple[float, float]] = None,
        max_tolerance: Optional[Tuple[float, float]] = None,
        max_scatter: Optional[float] = None,
        minimum_root_weight: Optional[int] = None,
        weight_assignment_iterations: Optional[int] = None,
        allowed_skip_modules: Optional[int] = None,
        allow_cross_track: Optional[bool] = None,
        clone_ghost_killing: Optional[bool] = None
    ) -> None: ...
    
    def solve(self, event: Event) -> List[Track]: ...
    
    def solve_parallel(self, event: List[Event]) -> List[Track]: ...

class SearchByTripletTrie:
    def __init__(
        self,
        max_scatter: Optional[float] = None,
        min_strong_track_length: Optional[int] = None,
        allowed_missed_modules: Optional[int] = None
    ) -> None: ...
    
    def solve(self, event: Event) -> List[Track]: ...
    
    def solve_parallel(self, event: List[Event]) -> List[Track]: ...

"""
Validation methods
"""
def validate_print(py_events: List[Dict[str, Any]], py_tracks: List[Track], verbose: bool) -> None: ...
def validate_efficiency(py_events: List[Dict[str, Any]], py_tracks: List[Track], particle_type: str) -> float: ...
def validate_to_json(py_events: List[Dict[str, Any]], py_tracks: List[Track], verbose: bool) -> Dict[str, Any]: ...
def validate_to_json_nested(py_events: List[Dict[str, Any]], py_tracks: List[Track], verbose: bool) -> Dict[str, Any]: ...