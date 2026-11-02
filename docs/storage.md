# Snapshot-lagring

Bibliotekets `storage::publish` gemmer et komplet `RunResult` pr. kilde og
`storage::read` læser det. Den kaldende kode angiver hentetidspunktet eksplicit,
også for en tom kørsel. CLI-kommandoen `hent` bruger lagringen og fjerner
beviseligt udløbne poster efter fuld validering før publicering. Slutdatoer
gælder hele dagen i Europe/Copenhagen; præcis lighed med en tidspunktgrænse
bevares. Ukendte slutgrænser bevares. Oprydning sker først ved næste
vellykkede hentning. En kørsel
skal være eksplicit komplet og uden kilde- eller valideringsfejl; ellers bevares
sidste snapshot. Dækningen er tekst fra kilden: Tjeks Bilka-katalogdækning er
komplet inden for det angivne scope, ikke nødvendigvis alle direkte tilbud.

`storage::data_dir` bruger først et eksplicit override, derefter
`TILBUD_DATA_DIR`, derefter på macOS `~/Library/Application Support/danske-tilbud-cli`
og på Linux `$XDG_DATA_HOME/danske-tilbud-cli` eller
`~/.local/share/danske-tilbud-cli`. Kilde-ID må kun bestå af ASCII-bogstaver,
cifre, bindestreg og understregning.

`<source>.jsonl` begynder med et objekt med `record_type: "snapshot"`,
`schema_version: 1`, `source`, `fetched_at`, `written_at`, `coverage`,
`complete` og `offer_count`. Efterfølgende linjer er kanoniske Offer v1-objekter;
der gemmes ingen rå svar eller beregnede enhedspriser. Et eksplicit komplet tomt
snapshot har alene manifestet med count 0. Tomme, beskadigede, ufuldstændige
eller ukendte formater afvises. Hver linje afsluttes med newline.

Læsning og skrivning tager samme ikke-blokerende fillås `<source>.lock`; låsefilen
kan blive liggende. Skrivning bruger en ejerskabsmærket midlertidig fil i samme
mappe, validerer, flusher og synkroniserer før atomisk rename. Ved normal fejl
ryddes kun egen midlertidige fil; nedbrud kan efterlade den. Der oprettes ingen
historik eller backups. Enhedspriser kan senere beregnes i den fælles motor.
