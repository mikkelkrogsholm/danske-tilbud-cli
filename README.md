<div align="center">
  <img src="assets/readme-hero.svg" alt="Tilbud — danske tilbud i din terminal" width="100%">
  <p><strong>Danske dagligvaretilbud, klar til både mennesker og programmer.</strong></p>
  <p>Hent lokale snapshots én gang, og søg i dem uden netværkskald.</p>
  <p><code>Rust</code> &nbsp; · &nbsp; <code>JSON som standard</code> &nbsp; · &nbsp; <code>3 kilder</code> &nbsp; · &nbsp; <code>14 Tjek-kæder</code></p>
  <p><a href="#kom-godt-i-gang">Kom godt i gang</a> &nbsp; · &nbsp; <a href="#kommandoer">Kommandoer</a> &nbsp; · &nbsp; <a href="#kilder-og-dækning">Kilder og dækning</a> &nbsp; · &nbsp; <a href="#udvikling">Udvikling</a></p>
</div>

---

## Kom godt i gang

Du skal have Rust og Cargo på `PATH`. Klon projektet og installér CLI'et:

```sh
git clone https://github.com/mikkelkrogsholm/danske-tilbud-cli.git
cd danske-tilbud-cli
cargo install --locked --path .

tilbud kilder
tilbud hent --kilde tjek
tilbud find kaffe --butik netto --human
```

Hentning kræver netværk; `find` søger efterfølgende i de lokale snapshots. Cargo installerer normalt binæren i `~/.cargo/bin`, som skal være på `PATH`. En alternativ destination kan vælges med `cargo install --locked --path . --root /sti/til/installationsmappe`; binæren ligger i dens `bin`-mappe.

### Til scripts og agenter

JSON er standard på stdout. Her søges efter begge ord, og resultaterne hentes i sider:

```sh
tilbud find "økologisk mælk" --compact --limit 10
tilbud find "økologisk mælk" --compact --limit 10 --offset 10
```

Svaret indeholder `total`, `offset`, `limit` og `has_more`. `--compact` giver et fladt JSON-format med identitet, butik, beskrivelse og øvrige vilkår; det har samme metadata og udvalg som fuldt JSON. Brug `--human` for en læsbar visning af samme side; det kan ikke kombineres med `--compact`.

```sh
tilbud find "økologisk mælk" --limit 10 --human
tilbud find kaffe --kilde bilkatogo --include-unknown --human
```

## Kommandoer

| Kommando | Formål |
| --- | --- |
| `tilbud kilder` | Vis kilder, metadata og seneste snapshots. |
| `tilbud beskriv tjek` | Vis metadata, kendte kæde-ID'er og snapshot for en kilde. Brug også `rema` eller `bilkatogo`. |
| `tilbud hent --kilde tjek` | Hent en komplet kilde og erstat dens snapshot ved succes. |
| `tilbud hent --alle` | Hent alle kilder; hver fuldført kilde publiceres uafhængigt. |
| `tilbud find kaffe --butik netto` | Søg lokalt; afgræns eventuelt med `--kilde`, `--butik`, `--limit` og `--offset`. |

Alle kommandoer returnerer som standard et JSON-objekt med `schema_version`, `command` og `data`. Tilføj `--human` efter kommandoen for læsbar, statisk tekst. Fejl giver en ikke-nul exitkode og diagnosticering på stderr; delvise resultater vises på stdout. Ved `hent --alle` indeholder resultatet status for hver kilde. Vælg præcis én af `--alle` og `--kilde` ved hentning.

`--data-dir /sti/til/data` placeres efter kommandoen og tilsidesætter `TILBUD_DATA_DIR` og platformens standardmappe. Se [lagerkontrakten](docs/storage.md) for placeringer og format.

## Kilder og dækning

| Kilde | Hvad hentes? | Vigtigt for resultatet |
| --- | --- | --- |
| [Tjek](docs/tjek.md) | Tilbud for 14 danske kæder. | Direkte feed for de fleste kæder; dækningen er ikke en garanti for alle tilbud. Bilka rammer en grænse ved 1.000 og bruger katalog-fallback, som udelader selvstændige direkte tilbud ud over grænsen. |
| [REMA 1000](docs/rema.md) | Det offentlige produktkatalog. | Ikke dokumentation for lager i en bestemt butik. Søgning medtager aktuelle kampagneposter, ikke katalogpriser. |
| [BilkaToGo](docs/bilkatogo.md) | Tilbudsavisen for varer i sortiment og på lager i valgt scope; CLI bruger leveringslager `1653`. | Ikke hele kataloget eller alle fysiske butikker. Perioder er ukendte, så brug `--include-unknown`; en annonceret vare er ikke nødvendigvis nedsat. |

<details>
<summary>Se de 14 Tjek-kæder og deres ID'er</summary>

`rema1000`, `netto`, `lidl`, `foetex`, `bilka`, `365discount`, `meny`, `superbrugsen`, `kvickly`, `brugsen`, `spar`, `loevbjerg`, `min-koebmand`, `abc-lavpris`.

Brug ID'et med f.eks. `--butik netto`. Se [Tjek-kilden](docs/tjek.md) for metode og observeret dækning.
</details>

## Sådan behandles data

Et snapshot erstattes kun efter en fuldført hentning af den enkelte kilde. Ved næste vellykkede hentning fjernes beviseligt udløbne poster fra det nye snapshot; fremtidige poster og poster uden kendt slutgrænse bevares. `expired_removed` viser antallet. Der sker ingen automatisk oprydning uden hentning.

Søgning viser som standard kun beviseligt aktuelle tilbud. `--include-unknown` medtager også ukendt gyldighed, men ikke kendt fremtidige eller udløbne tilbud. Kontroller altid pris, periode og betingelser hos butikken: kilderne kan ændre sig, og et lokalt snapshot kan være forældet.

<details>
<summary>Søgning, sortering og fejl i detaljer</summary>

`find` søger i alle tilgængelige snapshots, medmindre `--kilde` vælger ét. Manglende snapshots springes over; ulæselige snapshots giver delvise resultater og ikke-nul exitkode. `--butik` læser kun kilder, der understøtter kæden. Tilbud fra forskellige kilder bevares separat, også ved samme navn eller EAN.

Søgeteksten deles ved whitespace. Alle ord skal forekomme som delstrenge i navn og/eller beskrivelse; søgningen ignorerer store og små bogstaver og ordstilling, men bevarer danske tegn. Der bruges hverken fuzzy matching eller synonymer, og tom søgetekst afvises. Sammenhængende frase i navnet rangeres først, derefter alle ord i navnet og til sidst match, der kræver beskrivelsen. Inden for samme relevans sorteres stabilt efter navn, kæde, kilde-ID og hele posten.

Datoer bruger Europe/Copenhagen, og slutdatoen er inklusiv. Ved præcis lighed med en tidspunktgrænse er gyldigheden ukendt, medmindre den anden grænse afgør den. Resultaterne indeholder original gyldighed, gyldighedsstatus og enhedspris, når den kan beregnes. `total` er antal match før pagination; `--limit` er positiv (standard 20), `--offset` er et ikke-negativt heltal (standard 0), og et offset efter sidste match giver en tom side. `limit` gælder samlet. `sources` viser hentetid og status for hvert snapshot; en fælles hentetid angives kun ved valg af én kilde.
</details>

## Dokumentation

- [Tilbudskontrakt og JSON-eksempler](docs/offer-v1.md) · [Lagerkontrakt og datamapper](docs/storage.md)
- [Tjek](docs/tjek.md) · [REMA 1000](docs/rema.md) · [BilkaToGo](docs/bilkatogo.md) — kildebegrænsninger og tredjepartsreferencer
- [Lokal verifikation](docs/verification.md) · [Søgeperformance](docs/search-performance.md)
- [Cargo.toml](Cargo.toml) og [Cargo.lock](Cargo.lock) viser Rust-afhængigheder og låste versioner; deres licenser følger de enkelte pakker.

Projektet deklarerer Rust 1.91 som minimum, men versionen er ikke afprøvet; den dokumenterede verifikation brugte Rust 1.99.1. Den kompilerede CLI er et selvstændigt Rust-program og kræver hverken Cargo, Graphify eller OpenWiki ved kørsel. Graphify og OpenWiki er udviklingsværktøjer. CLI-argumenter og hjælp håndteres af `usage-rs`.

## Udvikling

Kør testwrapperen fra projektets rod:

```sh
./scripts/test.sh
./scripts/test.sh --lib -- --nocapture
```

Cargo skal være på `PATH` eller angives med `CARGO=/absolut/sti/til/cargo ./scripts/test.sh`. Wrapperen isolerer og rydder sit eget build-output og sine testdata efter normal afslutning, fejl og håndterede signaler. SIGKILL, nedbrud og strømsvigt kan efterlade midlertidige filer.

<details>
<summary>Diskforbrug, testdata og Codex-worktrees</summary>

Cargo-profilerne slår debuginformation og incremental compilation fra for dev/test, også for build-afhængigheder. Det sparer plads mod ekstra kompileringstid. Wrapperen opretter en egen mappe med `mktemp` under `TMPDIR` eller `/tmp`; shellens signalhåndtering kan vente på den aktive Cargo-kommando. Der er ingen tidsstyret oprydning eller baggrundsjob. Argumenter sendes til `cargo test`, mens `--target-dir` er reserveret af wrapperen. Cargo understøtter ikke shell-aliaser i `config.toml`.

Nye tests skal placere midlertidige data under `TILBUD_TEST_TMPDIR` eller `std::env::temp_dir()` og vente på egne underprocesser. Versionerede fixtures bevares. Almindelig `cargo test` omgår wrapperen og kan efterlade build-output og testdata. Wrapperen rydder ikke globale Cargo-cacher eller andre builds.

Vælg **Worktrunk** som lokalt miljø i Codex; det kører `/opt/homebrew/bin/wt hook post-start --foreground` synkront. Opret, tilknyt og arkivér Codex-worktrees gennem Codex. Miljøvalget udføres ikke nødvendigvis automatisk for værktøjsoprettede opgaver. Projekt-hooks og cachekloning er fravalgt for at spare diskplads; der er ingen `.worktreeinclude`, delte skrivbare build-mapper eller automatisk oprydning ved worktree-livscyklus.
</details>
