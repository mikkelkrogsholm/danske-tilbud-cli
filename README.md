# Tilbud

Forberedelse til et dansk dagligvare-CLI i ren Rust. Projektet har endnu ingen
`Cargo.toml` eller programkode; testwrapperen kan først køre projektets tests,
når en Cargo-pakke er oprettet.

## Pladsbesparende udvikling

Cargo-konfigurationen slår debuginformation og incremental compilation fra for
dev/test, også for build-afhængigheder. Mindre diskforbrug koster ekstra
kompileringstid ved gentagne builds.

Kør fra projektets rod, når Cargo-pakken findes:

```sh
./scripts/test.sh
./scripts/test.sh --lib -- --nocapture
```

Cargo skal være på `PATH`, eller angives som
`CARGO=/absolut/sti/til/cargo ./scripts/test.sh`. Argumenter sendes videre til
`cargo test`; `--target-dir` reserveres af wrapperen. Cargo understøtter ikke
shell-aliaser i `config.toml`, så wrapperen køres direkte.

Hver kørsel får sin egen mappe via `mktemp` under operativsystemets tempmappe
(`TMPDIR`, ellers `/tmp`). Både build-output og testdata fjernes efter normal
afslutning, fejl og håndterede HUP/INT/TERM-signaler. Shellens signalhåndtering
kan vente på den aktive Cargo-kommando. SIGKILL, nedbrud og strømsvigt kan
efterlade mappen; der er ingen tidsstyret oprydning eller baggrundsjob.

Fremtidige tests skal lægge midlertidige data under `TILBUD_TEST_TMPDIR` eller
`std::env::temp_dir()` (som følger `TMPDIR`), og vente på egne underprocesser.
Versionerede fixtures bevares. Almindelig `cargo test` omgår wrapperen og kan
efterlade både build-output og testdata. Globale Cargo-cacher og andre builds
ryddes aldrig af wrapperen.

## Worktrunk og Codex

Vælg **Worktrunk** som lokalt miljø i Codex. Miljøet kører
`/opt/homebrew/bin/wt hook post-start --foreground` synkront. Opret, tilknyt og
arkivér Codex-worktrees gennem Codex. Miljøvalget udføres ikke nødvendigvis
automatisk for værktøjsoprettede opgaver.

Der er ingen projekt-hooks eller cachekloning tilmeldt: projektet er let, og
lavt diskforbrug prioriteres. Ingen `.worktreeinclude`, delte skrivbare
build-mapper eller automatisk oprydning ved worktree-livscyklus er tilføjet.
