# Graph Report - tilbud  (2026-11-02)

## Corpus Check
- Code-only extract: 13 kildefiler; dokumenter og wiki er udeladt.

## Summary
- 172 nodes · 371 edges · 12 communities (7 shown, 5 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 5 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1b29c68e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Offer contract and tests
- Batch normalization tests
- Tjek fetch and mapping
- Tjek adapter tests
- Normalization and validation
- Source and offer types
- Tjek adapter and smoke
- Isolated test workflow
- CLI entry point
- Cargo package

## God Nodes (most connected - your core abstractions)
1. `RawOffer` - 17 edges
2. `Offer` - 15 edges
3. `TjekAdapter` - 15 edges
4. `SourceMetadata` - 13 edges
5. `normalize_batch()` - 12 edges
6. `adapter()` - 12 edges
7. `TimeBoundary` - 11 edges
8. `Membership` - 11 edges
9. `SourceBatch` - 11 edges
10. `fixture_server()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `dedup_is_order_independent_and_keeps_store_and_period_distinct()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `fractional_ore_noise_has_a_small_exact_tolerance()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `invalid_records_are_visible_and_do_not_suppress_valid_records()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `pack_ranges_missing_and_zero_price()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `unknown_heading_preserves_the_identified_offer()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs

## Import Cycles
- None detected.

## Communities (12 total, 5 thin omitted)

### Community 0 - "Offer contract and tests"
Cohesion: 0.10
Nodes (17): Membership, NotRequired, Required, Unknown, Offer, PieceCount, Exact, Range (+9 more)

### Community 1 - "Batch normalization tests"
Cohesion: 0.15
Nodes (15): normalize_batch(), SourceAdapter, SourceCapabilities, SourceMetadata, batch(), block_on(), dedup_is_order_independent_and_keeps_store_and_period_distinct(), fractional_ore_noise_has_a_small_exact_tolerance() (+7 more)

### Community 2 - "Tjek fetch and mapping"
Cohesion: 0.14
Nodes (9): boundary(), CHAINS, LIMIT, map_offer(), MAX_PAGES, nullable_string(), number(), parse_time() (+1 more)

### Community 3 - "Tjek adapter tests"
Cohesion: 0.22
Nodes (16): adapter(), bad_row_is_reported_but_following_offer_is_kept(), capped_prefix(), catalog_pages_are_traversed_before_offer_pages(), catalog_store_is_preserved_and_dates_filter_in_copenhagen(), empty_catalogs_are_not_claimed_as_offer_coverage(), filtering_store_date_and_country(), fixture_server() (+8 more)

### Community 4 - "Normalization and validation"
Cohesion: 0.15
Nodes (12): boundary(), calculate_unit_price(), exact_product(), identity(), InvalidRecord, money(), normalize(), normalize_quantity() (+4 more)

### Community 5 - "Source and offer types"
Cohesion: 0.14
Nodes (14): PriceKind, Catalog, Offer, QuantityBasis, PerPiece, Total, Store, FetchOptions (+6 more)

### Community 7 - "Isolated test workflow"
Cohesion: 0.40
Nodes (3): test.sh script, TILBUD_TEST_TMPDIR, TMPDIR

## Knowledge Gaps
- **20 isolated node(s):** `NotRequired`, `Required`, `Unknown`, `Exact`, `Range` (+15 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 67 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `TjekAdapter` connect `Tjek adapter and smoke` to `Batch normalization tests`, `Tjek fetch and mapping`, `Tjek adapter tests`?**
  _High betweenness centrality (0.128) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `normalize_batch()` (e.g. with `dedup_is_order_independent_and_keeps_store_and_period_distinct()` and `fractional_ore_noise_has_a_small_exact_tolerance()`) actually correct?**
  _`normalize_batch()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `NotRequired`, `Required`, `Unknown` to the rest of the system?**
  _20 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Offer contract and tests` be split into smaller, more focused modules?**
  _Cohesion score 0.10344827586206896 - nodes in this community are weakly interconnected._
- **Why does `Membership` connect `Offer contract and tests` to `Batch normalization tests`, `Tjek fetch and mapping`, `Tjek adapter tests`, `Source and offer types`?**
  _High betweenness centrality (0.128) - this node is a cross-community bridge._
- **Should `Tjek fetch and mapping` be split into smaller, more focused modules?**
  _Cohesion score 0.14492753623188406 - nodes in this community are weakly interconnected._
- **Why does `SourceMetadata` connect `Batch normalization tests` to `Tjek fetch and mapping`, `Normalization and validation`, `Source and offer types`, `Tjek adapter and smoke`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._