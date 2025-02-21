# **graph_dfs** 🚀

## 📌 Overview

In the **graph_dfs** approach, particle hits are first pre-ordered and then connected into segments that form the nodes of a directed graph. A depth-first search (DFS) is subsequently used to traverse the graph and build full tracks. The algorithm can be summarized in the following steps:

1. **Pre-ordering**: Sort hits in each detector module by the $x$-coordinate and assign a unique hit number.
2. **Candidate Generation**: For each hit $h$ in a module, determine a candidate range in preceding modules based on compatibility in the $x$-direction.
3. **Segment Construction**: Form segments between hits that are compatible in both $x$ and $y$ directions.
4. **Graph Formation**: Build a directed graph where nodes are segments and an edge from segment $s_0$ to $s_1$ exists if $s_1$ can continue $s_0$ (i.e., $s_0.h_1 = s_1.h_0$) and the combined segment passes a tolerance check.
5. **Weight Assignment and Root Detection**: Propagate weights through the graph such that for a segment $s$,
   ```math
   w(s) = \max_{s' \in \mathcal{C}(s)} \, w(s') + 1,
   ```
   where $\mathcal{C}(s)$ is the set of segments compatible with $s$. Segments with no incoming edges (or those not “dominated” by others) are marked as **root segments**.
6. **Depth-First Search (DFS)**: Starting from each root segment, perform DFS to extract the longest contiguous sequence of segments, yielding a candidate track.
7. **Pruning**: Apply clone and ghost killing to remove short or redundant tracks, ensuring uniqueness.

---

## 🛠 Logic

### 1. **Pre-ordering and Candidate Generation**

Let $ H = \{h_1, h_2, \dots, h_N\} $ denote the set of all hits ordered by module and sorted by $x$. Each hit $ h_i $ is represented as:
```math
h_i = (x_i, y_i, z_i), \quad \text{with a unique hit number } n_i.
```
For each hit $ h_i $ in module $ m $, define a candidate set $ C_i $ in an earlier module $ m' $ such that:
```math
C_i(m') = \{ h_j \in \text{module } m' \mid |x_j - x_i| < \alpha_x \cdot |z_j - z_i| \},
```
where $\alpha_x$ is the maximum allowed slope in $x$.

### 2. **Segment Construction**

A **segment** $ s $ is defined as an ordered pair of hits:
```math
s = (h_0, h_1),
```
where $ h_0 $ and $ h_1 $ satisfy:
```math
|x_1 - x_0| < \alpha_x \cdot |z_1 - z_0| \quad \text{and} \quad |y_1 - y_0| < \alpha_y \cdot |z_1 - z_0|.
```
Furthermore, a tolerance function $ \mathcal{T}(h_0, h_1, h_2) $ is defined to verify the consistency of a predicted hit $ h_2 $ with a segment:
```math
\begin{aligned}
t_x &= \frac{x_1 - x_0}{z_1 - z_0}, \quad t_y = \frac{y_1 - y_0}{z_1 - z_0}, \\
\hat{x}_2 &= x_0 + t_x \cdot (z_2 - z_0), \quad \hat{y}_2 = y_0 + t_y \cdot (z_2 - z_0), \\
\mathcal{T}(h_0, h_1, h_2) &= \left( |x_2 - \hat{x}_2| < T_x \right) \wedge \left( |y_2 - \hat{y}_2| < T_y \right) \wedge \left( \text{scatter} < S \right),
\end{aligned}
```
with $T_x, T_y$ being the tolerances and $S$ the maximum scatter allowed.

### 3. **Graph Formation**

Let $ \mathcal{S} $ be the set of all segments constructed. Define a directed edge $ s_0 \to s_1 $ if:
- The endpoint of $ s_0 $ matches the starting point of $ s_1 $:
  ```math
  s_0.h_1 = s_1.h_0,
  ```
- And the combined segments satisfy the tolerance condition:
  ```math
  \mathcal{T}(s_0.h_0, s_0.h_1, s_1.h_1) = \text{True}.
  ```
This defines the compatibility set:
```math
\mathcal{C}(s_0) = \{ s_1 \in \mathcal{S} \mid s_0 \to s_1 \}.
```

### 4. **Weight Assignment and Root Detection**

Weights are iteratively assigned to segments to reflect their ability to extend into longer tracks:
```math
w(s) = \max_{s' \in \mathcal{C}(s)} \, w(s') + 1, \quad \text{for a fixed number of iterations.}
```
A segment is designated as a **root segment** if it is not a continuation of any other segment, i.e., if no segment $ s' $ exists such that $ s' \to s $.

### 5. **Track Extraction via DFS**

For each root segment $ s_{\text{root}} $, the DFS explores paths:
```math
\text{DFS}(s_{\text{root}}) = \{ h_0, h_1, \dots, h_k \},
```
where the path is constructed by concatenating the hits along the chain of segments:
```math
\text{Track} = [s.h_0 \, \text{of root}] \oplus \bigoplus_{s \in \text{path}} s.h_1.
```
The DFS selects the longest or highest-weighted path as the candidate track.

### 6. **Pruning**

Finally, a pruning function is applied to eliminate:
- **Clones**: Duplicate tracks sharing a significant number of hits.
- **Ghosts**: Weak tracks (typically with 3 hits) that overlap with stronger tracks.
This ensures that the final track set $ \mathcal{T} $ consists of unique, high-confidence tracks.

---

## 🎯 Conclusion

The **graph_dfs** algorithm transforms the problem of track reconstruction into a graph search problem. By representing hit connections as segments and leveraging compatibility functions $ \mathcal{T} $, the method assigns weights and detects root segments from which tracks are extracted using DFS. Pruning further refines the result to yield a set of unique tracks, balancing sensitivity and specificity in the reconstruction process.

---
[Go Back](../readme.md)