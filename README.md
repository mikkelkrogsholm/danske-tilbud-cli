# Tilbud

Et dansk dagligvare-CLI med lokale snapshots fra Tjek. Søgning foretager
ingen netværkskald; hent først en komplet kilde. Kilden henter direkte tilbud for de fleste angivne kæder, ikke nødvendigvis alle tilbud;
Bilka bruger katalog-fallback, hvor selvstændige direkte tilbud ud over 1000 udelades. Se [kildens begrænsninger](docs/tjek.md).

```sh
cargo run -- kilder
cargo run -- beskriv tjek
cargo run -- hent --kilde tjek
cargo run -- find kaffe --butik netto --limit 20
cargo run -- find mælk --include-unknown --data-dir /sti/til/data
```

Alle kommandoer giver et JSON-objekt med `schema_version`, `command` og `data`
på stdout. Fejl giver en ikke-nul exitkode og diagnosticering på stderr.
`--data-dir` placeres efter kommandoen og tilsidesætter `TILBUD_DATA_DIR` og
platformens standardmappe, som beskrevet i [lagerkontrakten](docs/storage.md).
`kilder` og `beskriv` viser metadata, kendte kæde-ID'er og seneste snapshot,
hvis det findes. `hent` henter hele kilden og erstatter kun et eksisterende
snapshot efter en fuldført kørsel. `find` søger uden hensyn til store og små
bogstaver i navn og beskrivelse. Den viser kun tilbud, der er beviseligt
aktuelle; `--include-unknown` medtager også poster med ukendte datoer, men
ikke poster der vides at være fremtidige eller udløbne. Datoer bruger
Europe/Copenhagen og slutdatoen er inklusiv; tidspunkter sammenlignes præcist.
Resultater indeholder original gyldighed, gyldighedsstatus og enhedspris, når
den kan beregnes. `total` er antal match før `limit` (standard 20). Katalogpriser
udelades. `fetched_at` fortæller, hvornår snapshot blev hentet.

Den fælles serde-model og dokumenterede JSON-eksempler findes i
[tilbudskontrakten](docs/offer-v1.md). CLI-argumenter og hjælp håndteres af
`usage-rs`.

## Pladsbesparende udvikling

Cargo-konfigurationen slår debuginformation og incremental compilation fra for
dev/test, også for build-afhængigheder. Mindre diskforbrug koster ekstra
kompileringstid ved gentagne builds.

Kør fra projektets rod:

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
