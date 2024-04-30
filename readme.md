# `rehexed`
This crate is meant to process the output of `hexasphere`'s
icosahedron subdivision (aka `IcoSphere`) into an adjacency
list for use in instances where hexagonal tiles are needed.

Such examples include geometry generation, board algorithms
etc.

# Usage
Generate an icosphere subdivision:
```rs
use hexasphere::shapes::IcoSphere;

let sphere = IcoSphere::new(12, |_| {});
```
Accumulate its indices:
```rs
let indices = sphere.get_all_indices();
```
And then apply the one function:
```rs
let adjacency_list = rehexed::rehexed(&indices, sphere.raw_points.len());
```

