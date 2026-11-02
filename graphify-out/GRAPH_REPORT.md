# Graph Report - tilbud  (2026-11-02)

## Corpus Check
- Code-only extract: 17 kildefiler; dokumenter og wiki er udeladt. Den simple analyse slår to forbindelser sammen; graph.json bevarer 745 links.

## Summary
- 323 nodes · 743 edges · 14 communities (12 shown, 2 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7809dd1`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Offer models and normalization
- Snapshot storage and validation
- Tjek fetch and mapping
- CLI and source discovery
- REMA mapping and contract
- Source adapter dispatch
- REMA adapter tests
- Engine and validity tests
- CLI search and tests
- Tjek adapter tests
- Human output rendering
- Isolated test workflow
- Cargo package

## God Nodes (most connected - your core abstractions)
1. `RawOffer` - 19 edges
2. `Offer` - 18 edges
3. `SourceMetadata` - 18 edges
4. `TjekAdapter` - 16 edges
5. `SourceBatch` - 15 edges
6. `normalize_batch()` - 13 edges
7. `Membership` - 13 edges
8. `RemaAdapter` - 13 edges
9. `FetchOptions` - 13 edges
10. `SourceAdapter` - 12 edges

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

## Communities (14 total, 2 thin omitted)

### Community 0 - "Offer models and normalization"
Cohesion: 0.07
Nodes (39): boundary(), calculate_unit_price(), exact_product(), InvalidRecord, money(), normalize(), normalize_quantity(), NormalizedOffer (+31 more)

### Community 1 - "Snapshot storage and validation"
Cohesion: 0.09
Nodes (26): validate_offer(), concurrent_process_writer_is_rejected(), data_dir(), incomplete_and_injected_failure_preserve_bytes(), invalid(), lock(), lock_child(), Manifest (+18 more)

### Community 2 - "Tjek fetch and mapping"
Cohesion: 0.11
Nodes (14): RawBoundary, Date, Instant, Unknown, boundary(), CHAINS, LIMIT, map_offer() (+6 more)

### Community 3 - "CLI and source discovery"
Cohesion: 0.12
Nodes (15): Beskriv, Command, Beskriv, Find, Hent, Kilder, describe(), envelope() (+7 more)

### Community 4 - "REMA mapping and contract"
Cohesion: 0.11
Nodes (11): Membership, NotRequired, Required, Unknown, exact_boundary_and_unparseable_price_are_not_invented(), LIMIT, map_product(), MAX_PAGES (+3 more)

### Community 5 - "Source adapter dispatch"
Cohesion: 0.14
Nodes (10): Source, Rema, Tjek, RemaAdapter, FetchOptions, SourceAdapter, SourceBatch, SourceCapabilities (+2 more)

### Community 6 - "REMA adapter tests"
Cohesion: 0.14
Nodes (9): adapter(), ambiguous_price_and_missing_fields_stay_unknown(), errors_and_unsupported_options_are_incomplete(), page(), pages_map_current_prices_and_exact_quantities(), price(), product(), repeated_page_is_incomplete() (+1 more)

### Community 7 - "Engine and validity tests"
Cohesion: 0.16
Nodes (12): identity(), normalize_batch(), batch(), block_on(), dedup_is_order_independent_and_keeps_store_and_period_distinct(), fractional_ore_noise_has_a_small_exact_tolerance(), invalid_records_are_visible_and_do_not_suppress_valid_records(), metadata() (+4 more)

### Community 8 - "CLI search and tests"
Cohesion: 0.16
Nodes (8): human_narrow_search_keeps_long_names_conditions_and_unknown_prices(), human_uses_same_filtered_results_and_is_plain_when_piped(), local_search_filters_and_reports_snapshot(), missing_and_corrupt_snapshots_fail_usefully(), NEXT, rema_and_tjek_snapshots_coexist_and_catalog_does_not_leak_into_search(), success(), Temp

### Community 9 - "Tjek adapter tests"
Cohesion: 0.33
Nodes (16): adapter(), bad_row_is_reported_but_following_offer_is_kept(), capped_prefix(), catalog_pages_are_traversed_before_offer_pages(), catalog_store_is_preserved_and_dates_filter_in_copenhagen(), empty_catalogs_are_not_claimed_as_offer_coverage(), filtering_store_date_and_country(), fixture_server() (+8 more)

### Community 10 - "Human output rendering"
Cohesion: 0.28
Nodes (11): age(), boundary(), coverage(), description(), document(), fetch_uses_the_supplied_source_limitation(), limitation(), money() (+3 more)

### Community 11 - "Isolated test workflow"
Cohesion: 0.40
Nodes (3): test.sh script, TILBUD_TEST_TMPDIR, TMPDIR

## Knowledge Gaps
- **30 isolated node(s):** `Current`, `Expired`, `Future`, `Unknown`, `Exact` (+25 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 96 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SourceMetadata` connect `Source adapter dispatch` to `Offer models and normalization`, `Tjek fetch and mapping`, `CLI and source discovery`, `REMA mapping and contract`, `Engine and validity tests`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **What connects `Current`, `Expired`, `Future` to the rest of the system?**
  _30 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Offer models and normalization` be split into smaller, more focused modules?**
  _Cohesion score 0.0726764500349406 - nodes in this community are weakly interconnected._
- **Why does `Offer` connect `Offer models and normalization` to `Snapshot storage and validation`, `REMA mapping and contract`, `Engine and validity tests`?**
  _High betweenness centrality (0.072) - this node is a cross-community bridge._
- **Should `Snapshot storage and validation` be split into smaller, more focused modules?**
  _Cohesion score 0.09131205673758866 - nodes in this community are weakly interconnected._
- **Why does `Membership` connect `REMA mapping and contract` to `Offer models and normalization`, `Tjek fetch and mapping`, `Source adapter dispatch`, `REMA adapter tests`, `Engine and validity tests`, `Tjek adapter tests`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **Should `Tjek fetch and mapping` be split into smaller, more focused modules?**
  _Cohesion score 0.11397849462365592 - nodes in this community are weakly interconnected._