use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::event_model::hit::Hit;
use crate::event_model::event::Event;
use crate::event_model::track::Track;

#[pyclass]
#[derive(Clone, Debug)]
pub struct Segment {
    pub h0: Hit,
    pub h1: Hit,
    pub weight: i32,
    pub segment_number: usize,
    pub root_segment: bool,
}

impl Segment {
    pub fn new(h0: Hit, h1: Hit, seg_number: usize) -> Self { // Note the default settings in this struct... consider wheter this is valid or should be changed / exposed
        Segment {
            h0,
            h1,
            weight: 0,
            segment_number: seg_number,
            root_segment: false,
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Segment {}:\n h0: {:?}\n h1: {:?}\n Weight: {}",
            self.segment_number, self.h0, self.h1, self.weight
        )
    }
}

#[pyclass]
#[derive(Clone)]
pub struct GraphDFS {
    max_slopes: (f64, f64),
    max_tolerance: (f64, f64),
    max_scatter: f64,
    minimum_root_weight: i32,
    weight_assignment_iterations: i32,
    allowed_skip_modules: i32,
    allow_cross_track: bool,
    clone_ghost_killing: bool,
}

#[pymethods]
impl GraphDFS {
    #[new]
    pub fn new(
        max_slopes: Option<(f64, f64)>,
        max_tolerance: Option<(f64, f64)>,
        max_scatter: Option<f64>,
        minimum_root_weight: Option<i32>,
        weight_assignment_iterations: Option<i32>,
        allowed_skip_modules: Option<i32>,
        allow_cross_track: Option<bool>,
        clone_ghost_killing: Option<bool>,
    ) -> Self {
        GraphDFS {
            max_slopes: max_slopes.unwrap_or((0.7, 0.7)),
            max_tolerance: max_tolerance.unwrap_or((0.4, 0.4)),
            max_scatter: max_scatter.unwrap_or(0.4),
            minimum_root_weight: minimum_root_weight.unwrap_or(1),
            weight_assignment_iterations: weight_assignment_iterations.unwrap_or(2),
            allowed_skip_modules: allowed_skip_modules.unwrap_or(1),
            allow_cross_track: allow_cross_track.unwrap_or(true),
            clone_ghost_killing: clone_ghost_killing.unwrap_or(true),
        }
    }

    /// Solves the event using the DFS strategy and returns a list of Tracks.
    pub fn solve(&self, event: &Event) -> PyResult<Vec<Track>> {
        println!(
            "Invoking graph dfs with\n max slopes: {:?}\n max tolerance: {:?}\n max scatter: {}\n weight assignment iterations: {}\n minimum root weight: {}\n allow cross track: {}\n allowed skip modules: {}\n clone ghost killing: {}\n",
            self.max_slopes,
            self.max_tolerance,
            self.max_scatter,
            self.weight_assignment_iterations,
            self.minimum_root_weight,
            self.allow_cross_track,
            self.allowed_skip_modules,
            self.clone_ghost_killing
        );

        // Work on a copy of the event.
        let mut event_copy = event.clone();
        self.order_hits(&mut event_copy)?;
        let candidates = self.fill_candidates(&event_copy)?;
        let (mut segments, _outer_hit_segment_list, compatible_segments, populated_compatible_segments) =
            self.populate_segments(&event_copy, &candidates)?;
        // Optionally we can print compatible segments by uncommenting the following line:
        // self.print_compatible_segments(&segments, &compatible_segments, &populated_compatible_segments)?;
        self.assign_weights_and_populate_roots(&mut segments, &compatible_segments, &populated_compatible_segments)?;
        let root_segments: Vec<&Segment> = populated_compatible_segments
            .iter()
            .filter_map(|&segid| {
                let seg = &segments[segid];
                if seg.root_segment && seg.weight >= self.minimum_root_weight {
                    Some(seg)
                } else {
                    None
                }
            })
            .collect();

        let mut tracks: Vec<Track> = Vec::new();
        for root_seg in root_segments {
            let dfs_paths = self.dfs(root_seg, &segments, &compatible_segments)?;
            let mut selected_paths = dfs_paths.clone();
            // Sort paths by their length (number of hits) instead of comparing the hits directly.
            selected_paths.sort_by_key(|p| p.len());
            if !selected_paths.is_empty() {
                let selected = selected_paths[0].clone();
                let mut track_hits = vec![root_seg.h0.clone()];
                track_hits.extend(selected);
                tracks.push(Track::new(track_hits));
            }
        }
        if self.clone_ghost_killing {
            tracks = self.prune_short_tracks(tracks)?;
        }
        Ok(tracks)
    }
}

// Internal helper methods not exposed to Python.
impl GraphDFS {
    pub fn are_compatible_in_x(&self, hit0: &Hit, hit1: &Hit) -> PyResult<bool> {
        let hit_distance = (hit1.z - hit0.z).abs();
        let dxmax = self.max_slopes.0 * hit_distance;
        Ok((hit1.x - hit0.x).abs() < dxmax)
    }

    pub fn are_compatible_in_y(&self, hit0: &Hit, hit1: &Hit) -> PyResult<bool> {
        let hit_distance = (hit1.z - hit0.z).abs();
        let dymax = self.max_slopes.1 * hit_distance;
        Ok((hit1.y - hit0.y).abs() < dymax)
    }

    pub fn are_compatible(&self, hit0: &Hit, hit1: &Hit) -> PyResult<bool> {
        Ok(self.are_compatible_in_x(hit0, hit1)? && self.are_compatible_in_y(hit0, hit1)?)
    }

    // Checks for scatter tolerance.
    pub fn check_tolerance(&self, hit0: &Hit, hit1: &Hit, hit2: &Hit) -> PyResult<bool> {
        let dz = hit1.z - hit0.z;
        if dz == 0.0 {
            return Ok(false);
        }
        let td = 1.0 / dz;
        let tx = (hit1.x - hit0.x) * td;
        let ty = (hit1.y - hit0.y) * td;

        let dz2 = hit2.z - hit0.z;
        let x_prediction = hit0.x + tx * dz2;
        let dx = (x_prediction - hit2.x).abs();
        if dx >= self.max_tolerance.0 {
            return Ok(false);
        }
        let y_prediction = hit0.y + ty * dz2;
        let dy = (y_prediction - hit2.y).abs();
        if dy >= self.max_tolerance.1 {
            return Ok(false);
        }
        let scatter_num = dx * dx + dy * dy;
        let dz21 = hit2.z - hit1.z;
        if dz21 == 0.0 {
            return Ok(false);
        }
        let scatter_denom = 1.0 / dz21;
        let scatter = scatter_num * scatter_denom * scatter_denom;
        Ok(scatter < self.max_scatter)
    }

    // seg1 is expected to start where seg0 ends.
    pub fn are_segments_compatible(&self, seg0: &Segment, seg1: &Segment) -> PyResult<bool> {
        if seg0.h1 != seg1.h0 {
            println!("Warning: seg0.h1 and seg1.h0 are not the same");
            println!("{:?}", seg0.h1);
            println!("{:?}", seg1.h0);
        }
        self.check_tolerance(&seg0.h0, &seg0.h1, &seg1.h1)
    }

    // Preorders all hits in each module by x.
    // Instead of borrowing event.modules directly, we clone the modules to avoid conflict.
    pub fn order_hits(&self, event: &mut Event) -> PyResult<()> {
        let modules = event.modules.clone();
        for module in modules {
            let start = module.hit_start_index;
            let end = module.hit_end_index;
            event.hits[start..end].sort_by(|a, b| {
                a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        Ok(())
    }

    // Fills candidates for each hit.
    // Returns a vector (indexed by hit index) of HashMaps mapping a module index to a candidate range (start, end).
    pub fn fill_candidates(&self, event: &Event) -> PyResult<Vec<HashMap<usize, (i32, i32)>>> {
        let mut candidates: Vec<HashMap<usize, (i32, i32)>> = vec![HashMap::new(); event.number_of_hits];
        let cross_module_factor = if self.allow_cross_track { 1 } else { 2 };
        for module_index in (2..event.modules.len()).rev() {
            let s0 = &event.modules[module_index];
            let starting_module_index = module_index as i32 - cross_module_factor;
            let s0_hits = s0.hits()?; // assume this returns a slice of Hit
            for (offset, h0) in s0_hits.iter().enumerate() {
                // Compute the index of the hit in the global event.hits vector.
                let h0_index = s0.hit_start_index + offset;
                for missing_modules in 0..=self.allowed_skip_modules {
                    let target_module_index = starting_module_index - (missing_modules * cross_module_factor);
                    if target_module_index >= 0 {
                        let target_module_index_usize = target_module_index as usize;
                        let s1 = &event.modules[target_module_index_usize];
                        let mut begin_found = false;
                        let mut end_found = false;
                        candidates[h0_index].insert(target_module_index_usize, (-1, -1));
                        let s1_hits = s1.hits()?;
                        for (offset1, h1) in s1_hits.iter().enumerate() {
                            let h1_index = s1.hit_start_index + offset1;
                            let entry = candidates[h0_index].get_mut(&target_module_index_usize).unwrap();
                            if !begin_found && self.are_compatible_in_x(&h0, &h1)? {
                                *entry = (h1_index as i32, h1_index as i32 + 1);
                                begin_found = true;
                            } else if begin_found && !self.are_compatible_in_x(&h0, &h1)? {
                                *entry = (entry.0, h1_index as i32);
                                end_found = true;
                                break;
                            }
                        }
                        if begin_found && !end_found {
                            if let Some(_last_hit) = s1_hits.last() {
                                let entry = candidates[h0_index].get_mut(&target_module_index_usize).unwrap();
                                // Use the module’s end index as the end candidate.
                                *entry = (entry.0, s1.hit_end_index as i32);
                            }
                        }
                    }
                }
            }
        }
        Ok(candidates)
    }

    // Populates segments based on candidate ranges.
    // Returns a tuple containing:
    // - A vector of all segments.
    // - A vector (indexed by hit index) of segment indices.
    // - A vector (indexed by segment) of compatible segment indices.
    // - A vector of indices of segments that have at least one compatible segment.
    pub fn populate_segments(
        &self,
        event: &Event,
        candidates: &Vec<HashMap<usize, (i32, i32)>>,
    ) -> PyResult<(Vec<Segment>, Vec<Vec<usize>>, Vec<Vec<usize>>, Vec<usize>)> {
        let mut segments: Vec<Segment> = Vec::new();
        let mut outer_hit_segment_list: Vec<Vec<usize>> = vec![Vec::new(); event.hits.len()];
        for h0_index in 0..event.number_of_hits {
            if let Some(map) = candidates.get(h0_index) {
                for (&_module_number, &(start_candidate, end_candidate)) in map.iter() {
                    for h1_index in start_candidate..end_candidate {
                        let h1_index_usize = h1_index as usize;
                        if self.are_compatible_in_y(&event.hits[h0_index], &event.hits[h1_index_usize])? {
                            segments.push(Segment::new(
                                event.hits[h0_index].clone(),
                                event.hits[h1_index_usize].clone(),
                                segments.len(),
                            ));
                            outer_hit_segment_list[h1_index_usize].push(segments.len() - 1);
                        }
                    }
                }
            }
        }
        let mut compatible_segments: Vec<Vec<usize>> = vec![Vec::new(); segments.len()];
        for seg in &segments {
            // Here we assume that seg.h0 comes directly from event.hits, so we can find its index by matching IDs.
            let h0_hit_index = event
                .hits
                .iter()
                .position(|h| h.id == seg.h0.id)
                .unwrap_or(0);
            for &seg0_index in &outer_hit_segment_list[h0_hit_index] {
                let seg0 = &segments[seg0_index];
                if self.are_segments_compatible(seg0, seg)? {
                    compatible_segments[seg0.segment_number].push(seg.segment_number);
                }
            }
        }
        let populated_compatible_segments: Vec<usize> = compatible_segments
            .iter()
            .enumerate()
            .filter(|(_, seg_list)| !seg_list.is_empty())
            .map(|(i, _)| i)
            .collect();
        Ok((segments, outer_hit_segment_list, compatible_segments, populated_compatible_segments))
    }

    // Assigns weights to segments and marks root segments.
    pub fn assign_weights_and_populate_roots(
        &self,
        segments: &mut Vec<Segment>,
        compatible_segments: &Vec<Vec<usize>>,
        populated_compatible_segments: &Vec<usize>,
    ) -> PyResult<()> {
        for _ in 0..self.weight_assignment_iterations {
            for &seg0_index in populated_compatible_segments {
                if let Some(comp_list) = compatible_segments.get(seg0_index) {
                    let max_weight = comp_list
                        .iter()
                        .map(|&seg_num| segments[seg_num].weight)
                        .max()
                        .unwrap_or(0);
                    segments[seg0_index].weight = max_weight + 1;
                }
            }
        }
        for &seg0_index in populated_compatible_segments {
            segments[seg0_index].root_segment = true;
        }
        for &seg0_index in populated_compatible_segments {
            for &seg1_index in &compatible_segments[seg0_index] {
                segments[seg1_index].root_segment = false;
            }
        }
        Ok(())
    }

    /// Performs a depth-first search starting from a given segment.
    /// Returns all paths (as vectors of hits) found from this segment.
    pub fn dfs(
        &self,
        segment: &Segment,
        segments: &Vec<Segment>,
        compatible_segments: &Vec<Vec<usize>>,
    ) -> PyResult<Vec<Vec<Hit>>> {
        if compatible_segments[segment.segment_number].is_empty() {
            Ok(vec![vec![segment.h1.clone()]])
        } else {
            let mut result = Vec::new();
            for &segid in &compatible_segments[segment.segment_number] {
                let paths = self.dfs(&segments[segid], segments, compatible_segments)?;
                for mut path in paths {
                    let mut new_path = vec![segment.h1.clone()];
                    new_path.append(&mut path);
                    result.push(new_path);
                }
            }
            Ok(result)
        }
    }

    /// Prunes short tracks (clone/ghost killing).
    pub fn prune_short_tracks(&self, tracks: Vec<Track>) -> PyResult<Vec<Track>> {
        let mut used_hits: HashSet<i32> = HashSet::new();
        for t in &tracks {
            if t.hits.len() > 3 {
                for h in &t.hits {
                    // Use the hit's id instead of a non-existent hit_number.
                    used_hits.insert(h.id);
                }
            }
        }
        let pruned_tracks: Vec<Track> = tracks
            .into_iter()
            .filter(|t| {
                t.hits.len() > 3 || t.hits.iter().all(|h| !used_hits.contains(&h.id))
            })
            .collect();
        Ok(pruned_tracks)
    }

    /// (Debug) Prints compatible segments.
    pub fn print_compatible_segments(
        &self,
        segments: &Vec<Segment>,
        compatible_segments: &Vec<Vec<usize>>,
        populated_compatible_segments: &Vec<usize>,
    ) -> PyResult<()> {
        for &seg0_index in populated_compatible_segments {
            let seg0 = &segments[seg0_index];
            let comp_segments: Vec<String> = compatible_segments[seg0_index]
                .iter()
                .map(|&i| format!("{}", segments[i]))
                .collect();
            println!("{}\nis compatible with segments\n{:?}\n", seg0, comp_segments);
        }
        Ok(())
    }
}
