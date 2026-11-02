# Tilbud

Et dansk dagligvare-CLI med lokale snapshots fra Tjek, REMA 1000 og BilkaToGo. Søgning foretager
ingen netværkskald; hent først en komplet kilde. Tjek henter direkte tilbud for de fleste angivne kæder, ikke nødvendigvis alle tilbud;
Tjeks Bilka-dækning bruger katalog-fallback, hvor selvstændige direkte tilbud ud over 1000 udelades. Se [Tjeks begrænsninger](docs/tjek.md), [REMA-kildens begrænsninger](docs/rema.md) og [BilkaToGo-kildens begrænsninger](docs/bilkatogo.md).

## Lokal installation

Installér fra en lokal kopi af projektet med Rust og Cargo på `PATH`:

```sh
cd /sti/til/tilbud
cargo install --locked --path .
tilbud kilder
```

Cargo installerer normalt binæren i `~/.cargo/bin`, som skal være på `PATH`.
`cargo install --locked --path . --root /sti/til/installationsmappe` vælger
en anden destination; binæren ligger da i dens `bin`-undermappe. Projektet
deklarerer Rust 1.91 som minimum i `Cargo.toml`, men denne version er ikke
afprøvet; verificeringen nedenfor brugte Rust 1.99.1. Den kompilerede CLI er
et selvstændigt Rust-program og kræver hverken Cargo, Graphify eller OpenWiki
ved kørsel. Graphify og OpenWiki er udviklingsværktøjer.
Se [lokal verifikation](docs/verification.md) for de afprøvede kommandoer og
en dateret livekontrol.

Kildebeskrivelserne indeholder også tredjepartsreferencer og notitser om den
undersøgte kode: [Tjek](docs/tjek.md), [REMA](docs/rema.md) og
[BilkaToGo](docs/bilkatogo.md). Projektets
Rust-afhængigheder og låste versioner fremgår af [Cargo.toml](Cargo.toml) og
[Cargo.lock](Cargo.lock); deres egne licenser følger de enkelte pakker.

## Brug

```sh
tilbud kilder
tilbud beskriv tjek
tilbud beskriv bilkatogo
tilbud hent --kilde bilkatogo
tilbud find kaffe --kilde bilkatogo --include-unknown
tilbud beskriv rema
tilbud hent --kilde rema
tilbud find kirsebærsauce --kilde rema
tilbud hent --kilde tjek
tilbud hent --alle
tilbud find kaffe --butik netto --limit 20
tilbud find mælk --include-unknown --data-dir /sti/til/data
tilbud kilder --human
tilbud beskriv tjek --human
tilbud hent --kilde tjek --human
tilbud find kaffe --butik netto --human
```

Som standard giver alle kommandoer et JSON-objekt med `schema_version`, `command` og `data`
på stdout. Tilføj `--human` efter kommandoen for en læsbar, statisk visning af de samme data. Fejl giver en ikke-nul exitkode og diagnosticering på stderr; delvise resultater vises også på stdout.
`--data-dir` placeres efter kommandoen og tilsidesætter `TILBUD_DATA_DIR` og
platformens standardmappe, som beskrevet i [lagerkontrakten](docs/storage.md).
`kilder` og `beskriv` viser metadata, kendte kæde-ID'er og seneste snapshot,
hvis det findes. `hent` henter hele kilden og erstatter kun et eksisterende
snapshot efter en fuldført kørsel. Vælg præcis én af `--alle` og `--kilde`: med
`--alle` publiceres hver fuldført kilde uafhængigt, og et delvist resultat vises
på stdout med status for hver kilde, fejl på stderr og ikke-nul exitkode.
`find` søger som standard i alle tilgængelige snapshots uden netværkskald;
`--kilde` vælger én. Manglende snapshots springes over, mens ulæselige snapshots
giver et delvist resultat og ikke-nul exitkode. `--butik` filtrerer på kendte
kæde-ID’er og læser kun kilder, der understøtter kæden; `limit` gælder samlet. Tilbud fra forskellige kilder bevares
separat, også ved samme navn eller EAN. `find` søger uden hensyn til store og små
bogstaver i navn og beskrivelse. Den viser kun tilbud, der er beviseligt
aktuelle; `--include-unknown` medtager også poster med ukendte datoer, men
ikke poster der vides at være fremtidige eller udløbne. Datoer bruger
Europe/Copenhagen og slutdatoen er inklusiv. Ved præcis lighed med en
tidspunktgrænse er gyldigheden ukendt, medmindre den anden grænse afgør den.
Resultater indeholder original gyldighed, gyldighedsstatus og enhedspris, når
den kan beregnes. `total` er antal match før `limit` (standard 20). Katalogpriser
udelades. `sources` viser hentetid og status for hvert snapshot; en fælles
hentetid angives kun ved valg af én kilde.

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
