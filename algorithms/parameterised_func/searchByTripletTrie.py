import sys
sys.path.append("../")

import validator_lite as vl
import json
from event_model import *  # Assumes definitions for event, module, hit, track, etc.

class TrackFollowing:
    def __init__(self,
                 max_scatter=0.1,
                 merge_factor=2,
                 trie_size=26,
                 min_hits_for_flag=4,
                 max_hits_for_track=4,
                 forward_offset=2,
                 dummy_hit_params=(0, 0, 0, -1),
                 # Merging strategy functions:
                 merge_module_number_func=lambda m, mf: m.module_number / mf,
                 merge_z_func=lambda m: m.z,
                 merge_hit_start_index_func=lambda m: m.hit_start_index,
                 merge_hit_length_func=lambda first, last: last.hit_end_index - first.hit_start_index,
                 merge_global_hits_func=lambda m: m._module__global_hits):
        """
        Initialize the TrackFollowing solver.

        :param max_scatter: Maximum allowed scatter value.
        :param merge_factor: Number of modules to merge together.
        :param trie_size: Size of the trie (list) for compatible triplets.
        :param min_hits_for_flag: Number of hits in a track at which flagging should occur.
        :param min_hits_for_track: Minimum number of hits required to classify a track as valid.
        :param forward_offset: Offset used for slicing the merged module list in forwarding/seeding.
        :param dummy_hit_params: Parameters to create a dummy hit (used as a placeholder when no valid hit is found).
        :param merge_module_number_func: Function to calculate merged module number.
        :param merge_z_func: Function to calculate merged module z.
        :param merge_hit_start_index_func: Function to obtain the merged module hit start index.
        :param merge_hit_length_func: Function to calculate the length (number of hits) for the merged module.
        :param merge_global_hits_func: Function to obtain the global hits for the merged module.
        """
        self.max_scatter = max_scatter
        self.merge_factor = merge_factor
        self.trie_size = trie_size
        self.min_hits_for_flag = min_hits_for_flag
        self.min_hits_for_track = min_hits_for_track
        self.forward_offset = forward_offset
        self.dummy_hit_params = dummy_hit_params

        # Functions for merging module attributes.
        self.merge_module_number_func = merge_module_number_func
        self.merge_z_func = merge_z_func
        self.merge_hit_start_index_func = merge_hit_start_index_func
        self.merge_hit_length_func = merge_hit_length_func
        self.merge_global_hits_func = merge_global_hits_func

    def merge_module_groups(self, ev):
        """
        Merge modules in groups defined by merge_factor.
        """
        module_groups = []
        # Process modules in groups of self.merge_factor.
        for i in range(0, len(ev.modules), self.merge_factor):
            group = ev.modules[i:i+self.merge_factor]
            if len(group) < self.merge_factor:
                # If the last group is not complete, skip or handle as desired.
                continue
            m0 = group[0]
            m_last = group[-1]
            merged_module = module(
                self.merge_module_number_func(m0, self.merge_factor),
                self.merge_z_func(m0),
                self.merge_hit_start_index_func(m0),
                self.merge_hit_length_func(m0, m_last),
                self.merge_global_hits_func(m0)
            )
            module_groups.append(merged_module)
        return module_groups

    def calculate_scatter(self, h0, h1, h2):
        td = 1.0 / (h1.z - h0.z)
        txn = h1.x - h0.x
        tyn = h1.y - h0.y
        tx = txn * td
        ty = tyn * td

        dz = h2.z - h0.z
        x_prediction = h0.x + tx * dz
        y_prediction = h0.y + ty * dz
        dx = x_prediction - h2.x
        dy = y_prediction - h2.y

        return dx * dx + dy * dy

    def check_best_triplets(self, m0, m1, m2):
        best_triplets = []
        dummy = hit(*self.dummy_hit_params)
        for h0 in m0.hits():
            for h1 in m1.hits():
                best_h2 = dummy
                best_scatter = self.max_scatter
                for h2 in m2.hits():
                    scatter = self.calculate_scatter(h0, h1, h2)
                    if scatter < best_scatter:
                        best_h2 = h2
                        best_scatter = scatter
                if best_scatter < self.max_scatter:
                    best_triplets.append((h0, h1, best_h2, best_scatter))
        return best_triplets

    def generate_compatible_triplets(self, module_groups):
        compatible_triplets_trie = [None] * self.trie_size
        # Use the forward_offset parameter for slicing.
        for m0, m1 in zip(reversed(module_groups[self.forward_offset:]), reversed(module_groups[:-self.forward_offset])):
            compatible_triplets_module = {}
            compatible_triplets = []
            # m2 is chosen based on m1's module number.
            m2_index = m1.module_number - 1
            compatible_triplets += self.check_best_triplets(m0, m1, module_groups[m2_index])
            for h0, h1, h2, scatter in compatible_triplets:
                if h0 not in compatible_triplets_module:
                    compatible_triplets_module[h0] = {}
                compatible_triplets_module[h0][h1] = (h2, scatter)
            compatible_triplets_trie[m0.module_number] = compatible_triplets_module
        return compatible_triplets_trie

    def solve(self, ev):
        """
        Process a single event: merge modules, generate compatible triplets,
        perform seeding and forwarding, and return the final tracks.
        """
        # Merge modules based on the merge_factor.
        module_groups = self.merge_module_groups(ev)

        # Build a trie with compatible triplets.
        compatible_triplets_trie = self.generate_compatible_triplets(module_groups)

        flagged_hits = set()
        forwarding_tracks = []
        tracks = []
        weak_tracks = []

        dummy = hit(*self.dummy_hit_params)
        # Forwarding & Seeding: use forward_offset to determine slices.
        for m0, m1 in zip(reversed(module_groups[self.forward_offset:]), reversed(module_groups[:-self.forward_offset])):
            compatible_triplets_in_module = compatible_triplets_trie[m0.module_number]
            forwarding_next_step = []

            # Forwarding: extend existing tracks.
            for t in forwarding_tracks:
                h0 = t.hits[-2]
                h1 = t.hits[-1]
                found_h2 = False
                if t.missed_last_module or t.missed_penultimate_module:
                    best_h2 = dummy
                    best_scatter = self.max_scatter
                    # Use the original event modules (not merged) for recovery.
                    hits_m2 = ev.modules[m1.module_number - 1].hits()
                    for h2 in hits_m2:
                        scatter = self.calculate_scatter(h0, h1, h2)
                        if h2 not in flagged_hits and scatter < best_scatter:
                            best_h2 = h2
                            best_scatter = scatter
                    if best_h2.id != -1:
                        found_h2 = True
                        t.hits.append(best_h2)
                        flagged_hits.add(best_h2)
                        if len(t.hits) == self.min_hits_for_flag:
                            flagged_hits.update(t.hits[:self.min_hits_for_flag - 1])
                        forwarding_next_step.append(t)
                        t.missed_penultimate_module = t.missed_last_module
                        t.missed_last_module = False
                elif h0 in compatible_triplets_in_module and h1 in compatible_triplets_in_module[h0]:
                    (h2, scatter) = compatible_triplets_in_module[h0][h1]
                    found_h2 = True
                    t.hits.append(h2)
                    flagged_hits.add(h2)
                    if len(t.hits) == self.min_hits_for_flag:
                        flagged_hits.update(t.hits[:self.min_hits_for_flag - 1])
                    forwarding_next_step.append(t)
                if not found_h2:
                    t.missed_penultimate_module = t.missed_last_module
                    t.missed_last_module = True
                    if t.missed_penultimate_module:
                        if len(t.hits) >= self.min_hits_for_track:
                            tracks.append(t)
                        else:
                            weak_tracks.append(t)
                    else:
                        forwarding_next_step.append(t)

            forwarding_tracks = forwarding_next_step

            # Seeding: start new tracks.
            for h0 in compatible_triplets_in_module.keys():
                best_h1 = dummy
                best_h2 = dummy
                best_scatter = self.max_scatter
                for h1 in compatible_triplets_in_module[h0].keys():
                    (h2, scatter) = compatible_triplets_in_module[h0][h1]
                    if (h0 not in flagged_hits and 
                        h1 not in flagged_hits and 
                        h2 not in flagged_hits and 
                        scatter < best_scatter):
                        best_scatter = scatter
                        best_h1 = h1
                        best_h2 = h2
                if best_scatter < self.max_scatter:
                    forwarding_tracks.append(track([h0, best_h1, best_h2]))

        # Finalize track classification.
        for t in forwarding_tracks:
            if len(t.hits) >= self.min_hits_for_track:
                tracks.append(t)
            else:
                weak_tracks.append(t)

        for t in weak_tracks:
            if (t.hits[0] not in flagged_hits and
                t.hits[1] not in flagged_hits and
                t.hits[2] not in flagged_hits):
                tracks.append(t)

        return tracks

# Example usage.
if __name__ == "__main__":
    json_data_all_events = []
    all_tracks = []

    # Instantiate the solver with custom parameters.
    tf_solver = TrackFollowing(
        max_scatter=0.1,
        merge_factor=2,
        trie_size=26,
        min_hits_for_flag=4,
        min_hits_for_track=4,
        forward_offset=2,
        dummy_hit_params=(0, 0, 0, -1)
    )

    print("Processing events", end="")
    for event_number in range(0, 1):
        print(".", end="")
        sys.stdout.flush()
        with open("../velojson/" + str(event_number) + ".json", "r") as f:
            json_data = json.loads(f.read())
        ev = event(json_data)

        tracks = tf_solver.solve(ev)

        json_data_all_events.append(json_data)
        all_tracks.append(tracks)

    print("\nValidating solution")
    vl.validate_print(json_data_all_events, all_tracks)
