# Graph Report - tilbud  (2026-11-02)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 111 nodes · 216 edges · 12 communities (8 shown, 4 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 3 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4d10d350`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Batch processing and tests
- Offer data contract
- Normalization and validation
- Source adapter interface
- Membership contract tests
- Isolated test workflow
- CLI entry point
- Quantity representation
- Source time boundaries
- Cargo package

## God Nodes (most connected - your core abstractions)
1. `Offer` - 15 edges
2. `RawOffer` - 15 edges
3. `TimeBoundary` - 11 edges
4. `normalize_batch()` - 10 edges
5. `Quantity` - 10 edges
6. `SourceMetadata` - 10 edges
7. `Membership` - 9 edges
8. `SourceBatch` - 9 edges
9. `QuantityBasis` - 8 edges
10. `PieceCount` - 8 edges

## Surprising Connections (you probably didn't know these)
- `dedup_is_order_independent_and_keeps_store_and_period_distinct()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `invalid_records_are_visible_and_do_not_suppress_valid_records()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `pack_ranges_missing_and_zero_price()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `metadata()` --references--> `SourceMetadata`  [EXTRACTED]
  tests/engine.rs → src/source.rs
- `Offline` --references--> `SourceBatch`  [EXTRACTED]
  tests/engine.rs → src/source.rs

## Import Cycles
- None detected.

## Communities (12 total, 4 thin omitted)

### Community 0 - "Batch processing and tests"
Cohesion: 0.15
Nodes (11): normalize_batch(), RawOffer, SourceBatch, batch(), block_on(), dedup_is_order_independent_and_keeps_store_and_period_distinct(), invalid_records_are_visible_and_do_not_suppress_valid_records(), metadata() (+3 more)

### Community 1 - "Offer data contract"
Cohesion: 0.13
Nodes (17): Offer, PieceCount, Exact, Range, Price, PriceKind, Catalog, Offer (+9 more)

### Community 2 - "Normalization and validation"
Cohesion: 0.17
Nodes (12): boundary(), calculate_unit_price(), exact_product(), identity(), InvalidRecord, money(), normalize(), normalize_quantity() (+4 more)

### Community 3 - "Source adapter interface"
Cohesion: 0.23
Nodes (5): FetchOptions, SourceAdapter, SourceCapabilities, SourceMetadata, Offline

### Community 4 - "Membership contract tests"
Cohesion: 0.22
Nodes (4): Membership, NotRequired, Required, Unknown

### Community 5 - "Isolated test workflow"
Cohesion: 0.40
Nodes (3): test.sh script, TILBUD_TEST_TMPDIR, TMPDIR

### Community 7 - "Quantity representation"
Cohesion: 0.50
Nodes (3): Quantity, Exact, Range

### Community 8 - "Source time boundaries"
Cohesion: 0.50
Nodes (4): RawBoundary, Date, Instant, Unknown

## Knowledge Gaps
- **20 isolated node(s):** `Exact`, `Range`, `Catalog`, `Offer`, `PerPiece` (+15 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 53 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `RawOffer` connect `Batch processing and tests` to `Offer data contract`, `Normalization and validation`, `Source adapter interface`, `Membership contract tests`, `Quantity representation`, `Source time boundaries`?**
  _High betweenness centrality (0.087) - this node is a cross-community bridge._
- **What connects `Exact`, `Range`, `Catalog` to the rest of the system?**
  _20 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Offer data contract` be split into smaller, more focused modules?**
  _Cohesion score 0.13438735177865613 - nodes in this community are weakly interconnected._
- **Why does `Membership` connect `Membership contract tests` to `Batch processing and tests`, `Offer data contract`, `Source adapter interface`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Why does `TimeBoundary` connect `Offer data contract` to `Normalization and validation`, `Membership contract tests`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._