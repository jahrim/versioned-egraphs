# Versioned E-Graphs

Versioned E-Graphs for conditional equational reasoning.

A versioned e-graph is a semi-persistent data structure that stores many
equivalence relations over the same set of terms at once. Each relation lives in
its own *version*. Versions form a tree: a version inherits every term and
equality of its ancestors, and can extend them with its own assumptions without
affecting its ancestors or siblings.

## Installation

```toml
[dependencies]
versioned-egraphs = { git = "https://github.com/jahrim/versioned-egraphs" }
```

The library is imported as `veg`.

## Example

The example below adds terms to the root version, forks two versions with
different assumptions, queries equalities and disequalities in each version,
removes a version that became inconsistent, and e-matches a pattern against
different versions.

```rust
use veg::structures::egraph::ematching::machine::{CanEMatch, Pattern, Var};
use veg::structures::egraph::extensions::diegraph::{DiEGraph, DiEGraphView, DisequalityEdges};
use veg::structures::egraph::versioned::basic::VersionedEGraph;
use veg::structures::egraph::versioned::VersionedEGraph as _;
use veg::structures::egraph::{EGraph, EGraphView, ENode};
use veg::structures::versiontree::VersionTree;
use veg::util::id::SymbolIds;

fn main() {
    // A versioned e-graph tracking disequalities, with e-matching caches for
    // up to 16 versions.
    let mut egraph = VersionedEGraph::<DisequalityEdges>::with_ematching_caches(16);
    let root = VersionTree::ROOT_VERSION;

    // proj_mut: obtain a mutable view of a single version, which behaves as a
    // plain e-graph.
    // add: build the terms `x`, `y`, `f(x)` and `f(y)` in the root version.
    let [x, y, f] = ["x", "y", "f"].map(SymbolIds::from_string);
    let mut g = egraph.project_mut(root);
    let x = g.add(ENode::constant(x));
    let y = g.add(ENode::constant(y));
    let fx = g.add(ENode::application(f, vec![x]));
    let fy = g.add(ENode::application(f, vec![y]));

    // fork: create two versions that inherit everything from the root, one
    // assuming `x = y` and the other assuming `f(x) != f(y)`.
    let equal = egraph.branchout(root);
    let unequal = egraph.branchout(root);

    // union: in version `equal`, assume `x = y`. Rebuilding restores
    // congruence, so `f(x) = f(y)` follows.
    let mut g = egraph.project_mut(equal);
    g.union(x, y);
    g.rebuild();

    // proj: obtain a read-only view of a single version.
    // find: each version answers equality queries independently.
    let g = egraph.project(equal);
    assert_eq!(g.find(fx), g.find(fy));
    let g = egraph.project(root);
    assert_ne!(g.find(fx), g.find(fy));

    // disequalities: in version `unequal`, assume `f(x) != f(y)`.
    egraph.project_mut(unequal).disunion(fx, fy);
    assert!(egraph.project(unequal).unequal(fx, fy));
    assert!(egraph.project(unequal).is_consistent());

    // Exploring `x = y` under that assumption leads to a contradiction, since
    // congruence would imply `f(x) = f(y)`.
    let contradiction = egraph.branchout(unequal);
    let mut g = egraph.project_mut(contradiction);
    g.union(x, y);
    g.rebuild();
    assert!(!g.is_consistent());

    // remove: discard the inconsistent version and all of its subversions.
    egraph.remove_version(contradiction);
    assert!(!egraph.versioning().exists(contradiction));

    // ematching: search for `f(?a)` in each version. The root version has two
    // matching e-classes, `f(x)` and `f(y)`, while `equal` has merged them.
    let pattern: Pattern = "(f ?a)".parse().unwrap();
    assert_eq!(pattern.ematch(&egraph.project(root)).len(), 2);
    let g = egraph.project(equal);
    let matches = pattern.ematch(&g);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].eclass, g.find(fx));
    // `?a` is bound to the merged e-class of `x` and `y`.
    let a: Var = "?a".parse().unwrap();
    assert_eq!(g.find(matches[0].substs[0][a]), g.find(y));
}
```

By default, only leaf versions can be modified: once a version has
subversions, calling `project_mut` on it panics, so add terms and equalities
to its subversions instead. This behavior is controlled by
`egraph.flags_mut()`: setting `lock_superversions` to `false` allows modifying
superversions, and setting `propagate` to `true` also applies those changes to
their subversions.

## Features

- `pc-none`, `pc-split`, `pc-half` (default), `pc-full`: the path compression
  strategy used by the union-finds. Exactly one of them must be enabled:
  enabling none or several of them is not supported. To choose a strategy
  other than the default, disable the default features:

  ```toml
  versioned-egraphs = { git = "https://github.com/jahrim/versioned-egraphs", default-features = false, features = ["pc-full"] }
  ```
- `trace` and `trace-*`: instrument the data structures with
  [tracing](https://crates.io/crates/tracing) spans for debugging and
  profiling (see [Debugging](#debugging)).

## Debugging

### Inspecting an e-graph

Versioned e-graphs can be printed with `Debug`, rendered with
[Graphviz](https://graphviz.org/) through the `GraphViz` trait, and measured
through the `HasMemory` trait:

```rust
use veg::structures::egraph::versioned::basic::VersionedEGraph;
use veg::structures::egraph::versioned::VersionedEGraph as _;
use veg::structures::egraph::{EGraph, ENode};
use veg::structures::versiontree::VersionTree;
use veg::util::graphviz::GraphViz;
use veg::util::id::SymbolIds;
use veg::util::memory::HasMemory;

fn main() {
    let mut egraph = VersionedEGraph::<()>::default();
    let root = VersionTree::ROOT_VERSION;
    let [x, y] = ["x", "y"].map(|s| egraph.project_mut(root).add(ENode::constant(SymbolIds::from_string(s))));
    let v = egraph.branchout(root);
    egraph.project_mut(v).union(x, y);

    // Debug: print the state of a single version, or of the whole e-graph.
    println!("{:?}", egraph.project(v));

    // Graphviz: render the e-graph in DOT format, e.g. to view it with
    // `dot -Tsvg egraph.dot > egraph.svg`.
    let dot = format!("digraph {{\n{}\n}}", egraph.graphviz("my e-graph"));
    println!("{dot}");

    // Memory: measure the e-graph, or print a breakdown by field.
    println!("{}", egraph.memory());
    egraph.default_print_memory();
}
```

### Invariant checks

In debug builds, the versioned e-graph checks its internal invariants after
every `add`, `union` and `rebuild` on a version, and panics if one is violated.
Release builds skip these checks, so measure performance in release mode.

### Tracing and profiling

The `trace-*` features instrument the data structures with
[tracing](https://crates.io/crates/tracing) spans. Enable the ones for the
components you want to observe, for example:

```toml
versioned-egraphs = { git = "https://github.com/jahrim/versioned-egraphs", features = ["trace-egraph-versioned", "trace-machine"] }
```

The available features are `trace-unionfind`, `trace-egraph`,
`trace-disequalities` and `trace-machine` (e-matching), or `trace-all` for
everything. `trace-unionfind` and `trace-egraph` can also be narrowed down to
one implementation, e.g. `trace-egraph-versioned`.

The `veg::util::tracing` module (only available with the `trace-*` features)
then provides two ways to collect the traces. Only one of them can be
installed per program.

To log every traced call, with its arguments and return value, to standard
output:

```rust,ignore
veg::util::tracing::init_default_tracing_subscriber(true).unwrap(); // true: colored output
// ... run your code ...
```

To measure the total time spent in each traced operation:

```rust,ignore
let reporter = veg::util::tracing::init_profiler_tracing_subscriber().unwrap();
// ... run your code ...
reporter.report();
```

## License

Licensed under the [MIT License](https://github.com/jahrim/versioned-egraphs/blob/master/LICENSE).
This project contains code adapted from [egg](https://github.com/egraphs-good/egg);
see [THIRD-PARTY](https://github.com/jahrim/versioned-egraphs/blob/master/THIRD-PARTY).
