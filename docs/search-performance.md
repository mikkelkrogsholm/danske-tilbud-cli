# Søgning: måling af release-CLI

Målt 2. november 2026 på Apple M3 Max, arm64, macOS 26.6.2. En syntetisk
`tjek.jsonl` på 8.389.007 bytes indeholdt 12.000 validerede, aktuelle poster
med forskellige kilde-ID'er, fire roterende navne og tre beskrivelser. 3.000
poster matchede `kaffe`. Poster fulgte første eksempel i `offer-v1.md`, med
kilde `tjek`, kæde `netto` og gyldighed 2026-01-01 til 2027-12-31. Manifestet
angav 12.000 poster. Data var identiske i begge målinger.

Begge binærer blev bygget med `cargo build --release --locked`. Den oprindelige
revision var `2f6142d27b7af091a7a88de57f4b627dbc646f85`. Hver binær blev
kopieret til samme midlertidige sti og kørt med nøjagtigt denne kommando:

```sh
/tmp/tilbud-issue12-worker/run find kaffe --kilde tjek --data-dir /tmp/tilbud-issue12-worker
```

Tiden blev målt med Python `time.perf_counter` omkring en subprocess med stdout
fanget i en pipe: processtart, filindlæsning, validering, søgning og JSON-output
er med. To opvarmningskørsler efterfulgtes af 30 målinger; p95 er den 29.
sorterede observation. Målingerne blev udført sekventielt uden samtidige builds.

| Version | Median | p95 | stdout |
| --- | ---: | ---: | ---: |
| Før | 92,353 ms | 94,335 ms | 14.249 bytes |
| Efter | 93,570 ms | 95,583 ms | 14.276 bytes |

Den første førmåling, før kodeændring, gav median 93,133 ms og p95 98,108 ms
med 30 gentagelser. Eftermålingens ekstra bytes skyldes sidemetadata. Målingen
bruger varm filcache og syntetiske data, og er ikke en garanti for andre
maskiner eller reelle snapshots. Læsning og validering af alle 12.000 poster
sker fortsat ved hvert kald; koden deserialiserer hver post to gange og sorterer
alle match før pagination. Det er sandsynlige omkostninger, men ingen separat
profilering identificerer deres andel. Målingen viser ingen stor ændring i
svartiden og begrunder ikke et større lagrings- eller indekseringsarbejde her.

## Gentag målingen

Byg først den ønskede revision som release-binær, og kør derefter:

```sh
python3 scripts/benchmark-search.py /absolut/sti/til/tilbud
```

Hjælperen genererer 12.000 poster fra kontraktens første eksempel i sin egen
midlertidige mappe, kontrollerer at CLI'en finder 3.000 match, og måler samme
singleword-kald med to opvarmninger og 30 gentagelser. Den rapporterer
input- og outputbytes, median og p95 og sletter sine data og låsefiler efter
kørslen. Datoerne i det genererede snapshot beregnes fra kørselsdagen, så
antal bytes og resultaternes metadata ikke nødvendigvis er identiske med den
oprindelige måling. Anvend samme genererede bytes og belastningsforhold til en
strengt sammenlignelig før/efter-måling.
