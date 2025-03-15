from enum import Enum
class ReconstructionAlgorithms(Enum):
    TRACK_FOLLOWING = {
        "x_slope": (float, None),
        "y_slope": (float, None),
        "x_tol": (float, None),
        "y_tol": (float, None),
        "scatter": (float, None),
    }
    GRAPH_DFS = {
        "x_slope": (float, None),
        "y_slope": (float, None),
        "x_tol": (float, None),
        "y_tol": (float, None),
        "scatter": (float, None),
        "minimum_root_weight": (float, None),
        "weight_assignment_iterations": (int, None),
        "allowed_skip_modules": (list, None),
        "allow_cross_track": (bool, None),
        "clone_ghost_killing": (bool, None),
    }
    SEARCH_BY_TRIPLET_TRIE = {
        "scatter": (float, None),
        "min_strong_track_length": (int, None),
        "allowed_missed_modules": (int, None),
    }

    def get_config(self):
        return self.value.copy() # return a copy to avoid modifying the original dictionary
