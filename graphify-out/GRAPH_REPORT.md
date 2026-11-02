# Graph Report - tilbud  (2026-11-02)

## Corpus Check
- Code-only extract: 14 kildefiler; dokumenter og wiki er udeladt.

## Summary
- 266 nodes · 585 edges · 11 communities (8 shown, 3 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `6db75d2`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Offer models and normalization
- Snapshot storage and validation
- Source batches and engine tests
- CLI and source discovery
- Tjek fetch and mapping
- Tjek adapter tests
- CLI search and tests
- Isolated test workflow
- Cargo package

## God Nodes (most connected - your core abstractions)
1. `Offer` - 18 edges
2. `RawOffer` - 17 edges
3. `TjekAdapter` - 17 edges
4. `normalize_batch()` - 13 edges
5. `SourceMetadata` - 13 edges
6. `adapter()` - 12 edges
7. `Command` - 11 edges
8. `TimeBoundary` - 11 edges
9. `Membership` - 11 edges
10. `SourceBatch` - 11 edges

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

## Communities (11 total, 3 thin omitted)

### Community 0 - "Offer models and normalization"
Cohesion: 0.06
Nodes (46): boundary(), calculate_unit_price(), exact_product(), identity(), InvalidRecord, money(), normalize(), normalize_quantity() (+38 more)

### Community 1 - "Snapshot storage and validation"
Cohesion: 0.09
Nodes (27): RunResult, validate_offer(), concurrent_process_writer_is_rejected(), data_dir(), incomplete_and_injected_failure_preserve_bytes(), invalid(), lock(), lock_child() (+19 more)

### Community 2 - "Source batches and engine tests"
Cohesion: 0.11
Nodes (18): normalize_batch(), run(), FetchOptions, SourceAdapter, SourceBatch, SourceCapabilities, SourceMetadata, batch() (+10 more)

### Community 3 - "CLI and source discovery"
Cohesion: 0.10
Nodes (16): Beskriv, Command, Beskriv, Find, Hent, Kilder, describe(), envelope() (+8 more)

### Community 4 - "Tjek fetch and mapping"
Cohesion: 0.13
Nodes (10): boundary(), CHAINS, LIMIT, map_offer(), MAX_PAGES, nullable_string(), number(), parse_time() (+2 more)

### Community 5 - "Tjek adapter tests"
Cohesion: 0.24
Nodes (16): adapter(), bad_row_is_reported_but_following_offer_is_kept(), capped_prefix(), catalog_pages_are_traversed_before_offer_pages(), catalog_store_is_preserved_and_dates_filter_in_copenhagen(), empty_catalogs_are_not_claimed_as_offer_coverage(), filtering_store_date_and_country(), fixture_server() (+8 more)

### Community 6 - "CLI search and tests"
Cohesion: 0.15
Nodes (5): local_search_filters_and_reports_snapshot(), missing_and_corrupt_snapshots_fail_usefully(), NEXT, success(), Temp

### Community 8 - "Isolated test workflow"
Cohesion: 0.40
Nodes (3): test.sh script, TILBUD_TEST_TMPDIR, TMPDIR

## Knowledge Gaps
- **28 isolated node(s):** `Current`, `Expired`, `Future`, `Unknown`, `NotRequired` (+23 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 89 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Offer` connect `Offer models and normalization` to `Snapshot storage and validation`?**
  _High betweenness centrality (0.092) - this node is a cross-community bridge._
- **Are the 6 inferred relationships involving `normalize_batch()` (e.g. with `run()` and `dedup_is_order_independent_and_keeps_store_and_period_distinct()`) actually correct?**
  _`normalize_batch()` has 6 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Current`, `Expired`, `Future` to the rest of the system?**
  _28 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Offer models and normalization` be split into smaller, more focused modules?**
  _Cohesion score 0.06246799795186892 - nodes in this community are weakly interconnected._
- **Why does `TjekAdapter` connect `Tjek fetch and mapping` to `Source batches and engine tests`, `CLI and source discovery`, `Tjek adapter tests`?**
  _High betweenness centrality (0.090) - this node is a cross-community bridge._
- **Should `Snapshot storage and validation` be split into smaller, more focused modules?**
  _Cohesion score 0.08816326530612245 - nodes in this community are weakly interconnected._
- **Why does `Membership` connect `Offer models and normalization` to `Source batches and engine tests`, `Tjek fetch and mapping`, `Tjek adapter tests`, `Offer contract tests`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._