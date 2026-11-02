# Lokal verifikation 2. november 2026

Verificeret på macOS med Rust/Cargo 1.99.1. `Cargo.toml` deklarerer Rust 1.91,
men den version blev ikke afprøvet. Følgende kontroller blev kørt fra
projektroden efter de nødvendige Clippy-stilrettelser:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
./scripts/test.sh
cargo install --locked --path . --debug --root "$INSTALL_DIR"
```

Alle bestod; testwrapperen kørte 53 tests. Clippy fandt først nested `if` og
sorteringsstil på Rust 1.99.1; de blev rettet og kontrollen gentaget. Build og
installation brugte egne midlertidige mapper. Installationen blev afprøvet fra
et andet, tomt arbejdsbibliotek med en eksplicit, separat `--data-dir`:

```sh
"$INSTALL_DIR/bin/tilbud" kilder --data-dir "$DATA_DIR"
"$INSTALL_DIR/bin/tilbud" beskriv tjek --data-dir "$DATA_DIR"
"$INSTALL_DIR/bin/tilbud" beskriv rema --data-dir "$DATA_DIR"
"$INSTALL_DIR/bin/tilbud" beskriv bilkatogo --data-dir "$DATA_DIR"
"$INSTALL_DIR/bin/tilbud" hent --alle --data-dir "$DATA_DIR"
"$INSTALL_DIR/bin/tilbud" kilder --data-dir "$DATA_DIR" --human
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --limit 100
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --kilde tjek --limit 100
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --kilde rema --limit 100
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --kilde bilkatogo --limit 100
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --kilde bilkatogo --include-unknown --limit 100
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --kilde bilkatogo --include-unknown --limit 100 --human
"$INSTALL_DIR/bin/tilbud" find kaffe --data-dir "$DATA_DIR" --butik netto --limit 100
```

Alle kommandoer returnerede 0. JSON-output blev parset og kontrolleret for
schema, kilder, status, hentetid, kildefilter og kædefilter. `hent --alle`
publicerede tre komplette snapshots uden kildefejl eller afviste poster: Tjek
5.579, REMA 3.943 (271 tilbud), BilkaToGo 2.959 poster i scope 1653. Dette er
øjebliksbilleder, ikke forventede faste antal. Søgning efter kaffe gav 0
BilkaToGo-resultater som standard og 37 med `--include-unknown`, alle med
ukendt gyldighed og butik 1653. Den læsbare visning viste samme antal, ukendt
periode, butik og snapshot-alder; hvert snapshot havde en nylig hentetid.
Netto-filteret gav kun Tjek-resultater for Netto. Tjek-kildens historiske
14-kæde-afprøvning og dækningsforbehold står i [tjek.md](tjek.md).

Testwrapperens egne mapper samt den særskilte mappe med installation, build og
live-data blev fjernet. Miljøets `deny.machine_wipe`-regel afviste først
rekursiv sletning; målrettet oprydning lykkedes uden ændring af hooks. Globale
caches og brugerdata blev ikke ryddet.
