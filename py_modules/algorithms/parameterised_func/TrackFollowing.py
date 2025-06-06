from py_modules.event_model.event_model import *

class track_following:
  '''The classical solver.

  It sequentially traverses all module modules, marking
  hits as used in the way.
  '''
  def __init__(self, max_slopes=(0.7, 0.7), max_tolerance=(0.4, 0.4), max_scatter=0.4, min_track_length=3, min_strong_track_length=4):
    self.__max_slopes = max_slopes
    self.__max_tolerance = max_tolerance
    self.__max_scatter = max_scatter
    self.__min_track_length = min_track_length
    self.__min_strong_length = min_strong_track_length

    print("Instantiating track_following solver with parameters\n max slopes: %s\n max tolerance: %s\n\
 max scatter: %s\n min track hits: %s\n min strong track hits: %s\n" % \
 (self.__max_slopes, self.__max_tolerance, self.__max_scatter, min_track_length, min_strong_track_length))

  def are_compatible(self, hit_0, hit_1):
    hit_distance = abs(hit_1[2] - hit_0[2])
    dxmax = self.__max_slopes[0] * hit_distance
    dymax = self.__max_slopes[1] * hit_distance
    return abs(hit_1[0] - hit_0[0]) < dxmax and \
           abs(hit_1[1] - hit_0[1]) < dymax

  def check_tolerance(self, hit_0, hit_1, hit_2):
    td = 1.0 / (hit_1.z - hit_0.z)
    txn = hit_1.x - hit_0.x
    tyn = hit_1.y - hit_0.y
    tx = txn * td
    ty = tyn * td

    dz = hit_2.z - hit_0.z
    x_prediction = hit_0.x + tx * dz
    dx = abs(x_prediction - hit_2.x)
    tolx_condition = dx < self.__max_tolerance[0]

    y_prediction = hit_0.y + ty * dz
    dy = abs(y_prediction - hit_2.y)
    toly_condition = dy < self.__max_tolerance[1]

    scatterNum = (dx * dx) + (dy * dy)
    scatterDenom = 1.0 / (hit_2.z - hit_1.z)
    scatter = scatterNum * scatterDenom * scatterDenom

    scatter_condition = scatter < self.__max_scatter
    return tolx_condition and toly_condition and scatter_condition

  def solve(self, event):
    # We are searching for tracks
    # We will keep a list of used hits to avoid clones
    weak_tracks = []
    tracks      = []
    used_hits   = []

    # Start from the last module, create seeds and forward them
    for s0, s1, starting_module_index in zip(reversed(event.modules[3:]), reversed(event.modules[1:-2]), reversed(range(0, len(event.modules) - 3))):
      for h0 in [h0 for h0 in s0 if h0.id not in used_hits]:
        for h1 in [h1 for h1 in s1 if h1.id not in used_hits]:
          
          if self.are_compatible(h0, h1):
            # We have a seed, let's attempt to form a track
            # with a hit from the following three modules
            h2_found = False
            strong_track_found = False

            module_index_iter = -1
            for module_index in [sid for sid in reversed(range(starting_module_index-2, starting_module_index+1)) if sid >= 0]:
              for h2 in event.modules[module_index]:
                if self.check_tolerance(h0, h1, h2):
                  forming_track = track([h0, h1, h2])
                  h2_found = True
                  module_index_iter = module_index
                  break
              if h2_found:
                break

            # Continue with following modules - "forward" track
            missed_stations = 0
            if h2_found:
              while (module_index_iter >= 0 and missed_stations < 3):
                module_index_iter -= 1
                missed_stations   += 1
                for h2 in event.modules[module_index_iter]:
                  if self.check_tolerance(forming_track.hits[-2], forming_track.hits[-1], h2):
                    forming_track.add_hit(h2)
                    missed_stations = 0
                    break

              # Add track to list of tracks
              if len(forming_track.hits) == self.__min_track_length:
                # Track is a "weak track", we are not sure if it's noise or a clone
                weak_tracks.append(forming_track)

              elif len(forming_track.hits) >= self.__min_strong_length:
                # There is strong evidence it's a good track
                tracks.append(forming_track)
                used_hits += [h.id for h in forming_track.hits]
                strong_track_found = True

            if strong_track_found:
              break

    # Process weak tracks
    for t in weak_tracks:
      used_hits_in_weak_track = [h for h in t.hits if h.id in used_hits]
      if len(used_hits_in_weak_track) == 0:
        used_hits += [h.id for h in t.hits]
        tracks.append(t)

    return tracks
