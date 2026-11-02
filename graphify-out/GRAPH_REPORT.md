# Graph Report - tilbud  (2026-11-02)

## Corpus Check
- 19 files · ~14,151 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 378 nodes · 913 edges · 14 communities (12 shown, 2 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 9 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Tilbudsmodel og normalisering
- Snapshots og låsning
- CLI integrationstests
- Kildekontrakt og adaptere
- BilkaToGo produkter og priser
- CLI kommandoer
- Tjek mapping
- Testwrapper
- Pakke
- BilkaToGo adaptertests
- Motorens tests
- Læsbar præsentation
- Tjek adaptertests

## God Nodes (most connected - your core abstractions)
1. `SourceMetadata` - 23 edges
2. `RawOffer` - 22 edges
3. `SourceBatch` - 19 edges
4. `Offer` - 18 edges
5. `FetchOptions` - 16 edges
6. `SourceAdapter` - 16 edges
7. `TjekAdapter` - 16 edges
8. `BilkaToGoAdapter` - 15 edges
9. `Membership` - 14 edges
10. `normalize_batch()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `batch()` --references--> `RawOffer`  [EXTRACTED]
  tests/engine.rs → src/source.rs
- `raw()` --references--> `RawOffer`  [EXTRACTED]
  tests/engine.rs → src/source.rs
- `adapter()` --references--> `BilkaToGoAdapter`  [EXTRACTED]
  tests/bilkatogo.rs → src/bilkatogo.rs
- `adapter()` --references--> `RemaAdapter`  [EXTRACTED]
  tests/rema.rs → src/rema.rs
- `batch()` --references--> `SourceBatch`  [EXTRACTED]
  tests/engine.rs → src/source.rs

## Import Cycles
- None detected.

## Communities (14 total, 2 thin omitted)

### Community 0 - "Tilbudsmodel og normalisering"
Cohesion: 0.06
Nodes (48): InvalidRecord, NormalizedOffer, RunResult, UnitPrice, ValidityStatus, Offer, PieceCount, Price (+40 more)

### Community 1 - "Snapshots og låsning"
Cohesion: 0.09
Nodes (26): Manifest, OwnedTemp, Snapshot, TestDir, validate_offer(), concurrent_process_writer_is_rejected(), data_dir(), incomplete_and_injected_failure_preserve_bytes() (+18 more)

### Community 10 - "CLI integrationstests"
Cohesion: 0.26
Nodes (9): Temp, combined_search_preserves_sources_and_reports_partial_failure(), human_narrow_search_keeps_long_names_conditions_and_unknown_prices(), human_uses_same_filtered_results_and_is_plain_when_piped(), local_search_filters_and_reports_snapshot(), missing_and_corrupt_snapshots_fail_usefully(), NEXT, rema_and_tjek_snapshots_coexist_and_catalog_does_not_leak_into_search() (+1 more)

### Community 2 - "Kildekontrakt og adaptere"
Cohesion: 0.09
Nodes (14): BilkaToGoAdapter, Fake, Source, RemaAdapter, FetchOptions, SourceAdapter, SourceBatch, SourceMetadata (+6 more)

### Community 4 - "BilkaToGo produkter og priser"
Cohesion: 0.08
Nodes (20): Membership, amount(), map_product(), MAX_PAGES, PAGE_SIZE, RESPONSE_LIMIT, ROOT_LIMIT, SCRIPT_LIMIT (+12 more)

### Community 5 - "CLI kommandoer"
Cohesion: 0.11
Nodes (20): Beskriv, Command, Find, Hent, Kilder, Tilbud, Beskriv, Find (+12 more)

### Community 7 - "Tjek mapping"
Cohesion: 0.16
Nodes (10): TjekAdapter, boundary(), CHAINS, LIMIT, map_offer(), MAX_PAGES, nullable_string(), number() (+2 more)

### Community 11 - "Testwrapper"
Cohesion: 0.40
Nodes (3): test.sh script, TILBUD_TEST_TMPDIR, TMPDIR

### Community 3 - "BilkaToGo adaptertests"
Cohesion: 0.13
Nodes (21): adapter(), config(), missing_config_and_invalid_selected_price_fail_without_snapshot(), missing_store_never_uses_root_or_other_store_and_ambiguous_quantity_is_retained(), page(), pagination_scope_and_price_are_preserved(), refresh_once_on_rejection_and_stop_on_second_rejection(), rejects_drift_duplicates_truncation_and_malformed_data() (+13 more)

### Community 6 - "Motorens tests"
Cohesion: 0.15
Nodes (12): normalize_batch(), batch(), block_on(), dedup_is_order_independent_and_keeps_store_and_period_distinct(), fractional_ore_noise_has_a_small_exact_tolerance(), invalid_records_are_visible_and_do_not_suppress_valid_records(), major_and_minor_price_units_convert_without_float_or_fractional_ore(), metadata() (+4 more)

### Community 8 - "Læsbar præsentation"
Cohesion: 0.17
Nodes (12): age(), boundary(), coverage(), description(), document(), fetch_uses_the_supplied_source_limitation(), limitation(), money() (+4 more)

### Community 9 - "Tjek adaptertests"
Cohesion: 0.33
Nodes (16): adapter(), bad_row_is_reported_but_following_offer_is_kept(), capped_prefix(), catalog_pages_are_traversed_before_offer_pages(), catalog_store_is_preserved_and_dates_filter_in_copenhagen(), empty_catalogs_are_not_claimed_as_offer_coverage(), filtering_store_date_and_country(), fixture_server() (+8 more)

## Knowledge Gaps
- **33 isolated node(s):** `Current`, `Expired`, `Future`, `Unknown`, `Exact` (+28 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 110 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SourceMetadata` connect `Kildekontrakt og adaptere` to `Tilbudsmodel og normalisering`, `BilkaToGo produkter og priser`, `CLI kommandoer`, `Motorens tests`, `Tjek mapping`?**
  _High betweenness centrality (0.084) - this node is a cross-community bridge._
- **What connects `Current`, `Expired`, `Future` to the rest of the system?**
  _33 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Tilbudsmodel og normalisering` be split into smaller, more focused modules?**
  _Cohesion score 0.059673659673659674 - nodes in this community are weakly interconnected._
- **Why does `Membership` connect `BilkaToGo produkter og priser` to `Tilbudsmodel og normalisering`, `BilkaToGo adaptertests`, `Motorens tests`, `Tjek mapping`, `Tjek adaptertests`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Should `Snapshots og låsning` be split into smaller, more focused modules?**
  _Cohesion score 0.08843537414965986 - nodes in this community are weakly interconnected._
- **Why does `PriceKind` connect `Tilbudsmodel og normalisering` to `BilkaToGo adaptertests`, `BilkaToGo produkter og priser`, `CLI kommandoer`, `Motorens tests`, `Tjek mapping`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Should `Kildekontrakt og adaptere` be split into smaller, more focused modules?**
  _Cohesion score 0.09446693657219973 - nodes in this community are weakly interconnected._

## Integritetsdiagnose
- Kodeekstraktion: 19 filer, 378 noder og 916 rå links; analysens simple graf har 913 kanter. De tre ekstra parallelle links bevares i graph.json.
- Ingen manglende eller dangling endepunkter, selvsløjfer eller eksakte dubletter blev fundet. Extractorens diagnose viser 12 mulige suppressionssteder i producenten; udtrækket kan derfor ikke betegnes som dokumenteret tabsløst før disse er vurderet.
- Cargo.toml og src/lib.rs gav ingen symboler. Markdown, docs og OpenWiki er udelukket. Manifestets 19 kildehashes matcher kildefilerne; separat frisk AST-ekstraktion matchede node-id og linkantal.
