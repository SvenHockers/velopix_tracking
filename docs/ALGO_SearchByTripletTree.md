# **Merged-Triplet Algorithm** 🚀

## 📌 Overview

The final algorithm proceeds in several stages:

1. **Module Merging**: Merge detector modules two-by-two to simplify the hit space.
2. **Scatter Calculation**: For three hits $h_0, h_1, h_2$, compute a scatter value measuring the deviation of $h_2$ from the extrapolated line defined by $h_0$ and $h_1$.
3. **Triplet Generation**: For each pair $(h_0, h_1)$ from merged modules, select the best third hit $h_2$ (if any) that minimizes scatter below a threshold.
4. **Trie Construction**: Organize compatible triplets into a trie structure for fast lookup during track building.
5. **Track Formation**: Using the trie, extend partial tracks by:
   - **Forwarding**: Extend existing tracks by appending compatible hits.
   - **Seeding**: Start new tracks from candidate triplets.
6. **Track Classification and Pruning**: Classify tracks into strong and weak candidates based on their length, and prune duplicates or weak tracks.
7. **Validation**: Validate the reconstructed tracks using an external validation module.

---

## 🛠 Logic

### 1. **Module Merging**

Let the original detector modules be denoted as $M_0, M_1, M_2, \dots M_n$. We form merged modules by pairing:
```math
M_i^{\text{merged}} = \text{Merge}(M_{2i}, M_{2i+1})
```
This makes a simpler set of modules, with each merged module containing hits that all come one after the other.


### 2. **Scatter Calculation**

For three hits:
```math
h_0 = (x_0, y_0, z_0), \quad h_1 = (x_1, y_1, z_1), \quad h_2 = (x_2, y_2, z_2),
```
define the slopes:
```math
t_x = \frac{x_1 - x_0}{z_1 - z_0}, \quad t_y = \frac{y_1 - y_0}{z_1 - z_0}.
```
The predicted position of $h_2$ is:
```math
\hat{x}_2 = x_0 + t_x (z_2 - z_0), \quad \hat{y}_2 = y_0 + t_y (z_2 - z_0).
```
The scatter is computed as:
```math
\text{scatter}(h_0, h_1, h_2) = (x_2 - \hat{x}_2)^2 + (y_2 - \hat{y}_2)^2.
```
A triplet $(h_0, h_1, h_2)$ is considered **compatible** if:
```math
\text{scatter}(h_0, h_1, h_2) < S_{\max},
```
where $S_{\max}$ is the maximum allowed scatter.

### 3. **Triplet Generation and Trie Construction**

For a given merged module pair, for each candidate pair $(h_0, h_1)$ in modules $M_i$ and $M_j$ respectively, determine:
```math
h_2^* = \arg \min_{h_2 \in M_k} \; \text{scatter}(h_0, h_1, h_2)
```
subject to:
```math
\text{scatter}(h_0, h_1, h_2^*) < S_{\max}.
```
The resulting triplets are organized in a trie—a special kind of tree that enables fast lookups:
```math
\mathcal{T}: h_0 \rightarrow h_1 \rightarrow (h_2^*, \text{scatter})
```
This data structure allows rapid lookup of a compatible $h_2$ given a pair $(h_0, h_1)$.

### 4. **Track Formation: Forwarding and Seeding**

Tracks are constructed by two complementary strategies:

- **Forwarding**:  
  Given a partial track $T = [h_0, h_1, \dots, h_n]$ with the last two hits $(h_{n-1}, h_n)$, check the trie for a compatible hit:
  ```math
  \text{If } \mathcal{T}(h_{n-1})[h_n] \text{ exists, then append } h_{n+1} = h_2^* \text{ to } T.
  ```
  If no compatible hit is found, the algorithm may tolerate a missed module. After two consecutive misses, the track is finalized.

- **Seeding**:  
  Independently, new tracks are seeded from unused candidate triplets:
  ```math
  T_{seed} = [h_0, h_1, h_2^*]
  ```
  provided none of $h_0, h_1, h_2^*$ have been flagged as already used.

### 5. **Track Classification and Pruning**

Tracks are classified based on their length:
- **Strong Tracks**: $\lvert T \rvert \geq 4$  
- **Weak Tracks**: $\lvert T \rvert = 3$  
A pruning function removes duplicate or "ghost" tracks, ensuring that overlapping tracks do not contaminate the final set:
$$
\mathcal{T}_{\text{final}} = \{ T \mid T \text{ is unique and } (\lvert T \rvert \geq 4 \text{ or non-overlapping weak track}) \}.
$$

### 6. **Validation**

The final set of tracks is validated against the event data using a dedicated validation module:

```python
vl.validate_print(json_data_events, all_tracks)
```

---
[Go Back](../readme.md)
