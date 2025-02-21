# **track_following** 🚀

## 📌 Overview

When charged particles traverse the LHCb detector, they leave a series of hits across multiple modules. The **track_following** algorithm aims to chain these hits together into continuous trajectories (tracks). It does so by:

- **Seeding** potential tracks from pairs of hits.
- **Validating** these seeds by checking compatibility with additional hits.
- **Classifying** and **finalizing** the resulting tracks based on length and uniqueness.

---

## 🛠 How It Works

### 1. **Initialization and Parameters**

Let:
- $\alpha_x, \alpha_y$ be the **maximum slopes** in the $x$ and $y$ directions.
- $T_x, T_y$ be the **maximum tolerances** in the $x$ and $y$ directions.
- $S$ be the **maximum scatter** threshold.

These parameters control how the algorithm decides if hits can form a valid track:
- **Slopes** $\alpha_x, \alpha_y$ limit how steeply the track can curve between modules.
- **Tolerances** $T_x, T_y$ define how far a new hit can deviate from a predicted trajectory.
- **Scatter** $S$ ensures that small random fluctuations do not accumulate excessively along the track.

---

### 2. **Seeding with Hit Pairs**

Given two hits $h_0$ and $h_1$ with coordinates
```math
h_0 = (x_0, y_0, z_0), \quad h_1 = (x_1, y_1, z_1),
```
we say they form a **seed** if the following slope conditions hold:

```math
\begin{aligned}
&\left| x_1 - x_0 \right| < \alpha_x \cdot \Delta z, \\
&\left| y_1 - y_0 \right| < \alpha_y \cdot \Delta z,
\end{aligned}
```
where $\Delta z = \left| z_1 - z_0 \right|$.

If both conditions are satisfied, $(h_0, h_1)$ becomes a candidate for track formation.

---

### 3. **Validation and Extension**

#### 3.1 **Adding a Third Hit**

To confirm a seed, the algorithm searches for a third hit $h_2 = (x_2, y_2, z_2)$ in preceding modules such that the predicted position matches within tolerances. Specifically, it estimates the slopes in $x$ and $y$:

```math
\begin{aligned}
t_x &= \frac{x_1 - x_0}{z_1 - z_0}, \\
t_y &= \frac{y_1 - y_0}{z_1 - z_0}.
\end{aligned}
```

Using these slopes, it predicts the position of $h_2$ at $z_2$:
```math
\hat{x}_2 = x_0 + t_x \cdot (z_2 - z_0), \quad
\hat{y}_2 = y_0 + t_y \cdot (z_2 - z_0).
```

The third hit is considered valid if
```math
\begin{aligned}
&\left| x_2 - \hat{x}_2 \right| < T_x, \\
&\left| y_2 - \hat{y}_2 \right| < T_y, \\
&\text{scatter} < S,
\end{aligned}
```
where $\text{scatter}$ measures the squared distance $(\Delta x^2 + \Delta y^2)$ normalized by the spacing between hits.

#### 3.2 **Forward (Backward) Propagation**

Once a valid triplet $(h_0, h_1, h_2)$ is found, the algorithm continues through earlier modules (in decreasing $z$-order) to extend the track. It repeatedly applies the same **tolerance** and **scatter** checks to each new hit.

A **missed station** counter allows for up to three consecutive modules without a valid new hit before the track extension is halted.

---

### 4. **Classification and Uniqueness**

1. **Weak Track**: A track with exactly three hits.
   - Stored separately, as it could be incomplete or noise.

2. **Strong Track**: A track with four or more hits.
   - Deemed high-confidence and its hit IDs are marked as *used* to prevent duplicates.

Any weak track that does not share hits with a strong track is ultimately accepted in the final set of reconstructed tracks.

---

### 🔄 The Algorithm Flow: Step-by-Step

1. **Initialize**: Load $\alpha_x, \alpha_y$, $T_x, T_y$, and $S$.  
2. **Seed**: Form pairs of hits in consecutive modules and apply slope checks.  
3. **Validate**: Find a third hit via tolerance checks, confirming the seed.  
4. **Extend**: Propagate backward through modules, attaching compatible hits.  
5. **Classify**: Determine if a track is **weak** (3 hits) or **strong** (≥ 4 hits).  
6. **Finalize**:  
   - Mark strong tracks’ hits as used.  
   - Accept weak tracks that do not overlap with used hits.

---

## 🎯 Conclusion

By translating particle hits into geometric constraints, the **track_following** algorithm  links multiple detector modules into tracks. Its two-tier classification (weak vs strong).
Although the classification does make a distinction between weak vs strong tracks it doens't enforce this (i.e. remove weak or suppresses them).

---
[Go Back](../readme.md)