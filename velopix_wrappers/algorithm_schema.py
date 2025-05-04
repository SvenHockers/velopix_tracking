from enum import Enum
from typing import cast
from .custom_types import *

class ReconstructionAlgorithms(Enum):
    TRACK_FOLLOWING = {
        "x_slope": (float, None),
        "y_slope": (float, None),
        "x_tol": (float, None),
        "y_tol": (float, None),
        "scatter": (float, None),
    }
    GRAPH_DFS = cast(dict[str, tuple[type[float] | type[int] | type[bool], None]], {
        "x_slope": (float, None),  
        "y_slope": (float, None), 
        "x_tol": (float, None),    
        "y_tol": (float, None),    
        "scatter": (float, None),  
        "minimum_root_weight": (int, None),          
        "weight_assignment_iterations": (int, None),    
        "allowed_skip_modules": (int, None),           
        "allow_cross_track": (bool, None),              
        "clone_ghost_killing": (bool, None),
    })
    SEARCH_BY_TRIPLET_TRIE = cast(dict[str, tuple[type[float] | type[int], None]], {
        "scatter": (float, None),
        "min_strong_track_length": (int, None),
        "allowed_missed_modules": (int, None),
    })

    def get_config(self) -> ConfigType:
        return self.value.copy() # type: ignore
    
    def get_bounds(self) -> boundType:
        """
        Implemented bounds of each variable directly in the enums this way we can easly retrieve them from the optimalisation func
        """
        bounds: boundType = {}
        if self is ReconstructionAlgorithms.TRACK_FOLLOWING:
            bounds = {
                "x_slope": (0, 1),
                "y_slope": (0, 1),
                "x_tol": (0.4 , 0.8),
                "y_tol": (0.4, 0.8),
                "scatter": (0.0, 0.8),
            }
        elif self is ReconstructionAlgorithms.GRAPH_DFS:
            bounds = {
                "x_slope": (0, 1),
                "y_slope": (0, 1),
                "x_tol": (0.4, 0.4),
                "y_tol": (0.4, 0.8),
                "scatter": (0.0, 0.8),
                "minimum_root_weight": (0.0, 10.0),
                "weight_assignment_iterations": (1, 10),
                "allowed_skip_modules": (0, 6),
                # For booleans we dont need bounds 
                "allow_cross_track": None,
                "clone_ghost_killing": None,
            }
        elif self is ReconstructionAlgorithms.SEARCH_BY_TRIPLET_TRIE:
            bounds = {
                "scatter": (0, 1),
                "min_strong_track_length": (1, 20),
                "allowed_missed_modules": (0, 5),
            }
        return bounds.copy()
