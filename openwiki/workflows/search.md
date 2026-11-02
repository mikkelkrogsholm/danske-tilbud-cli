---
type: workflow
title: Søgning for agenter
description: Find tilbud i lokale snapshots med ordmatch, kompakt JSON og pagination, og vurder resultatets begrænsninger.
tags: [search, cli, pagination]
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T16:08:33.221Z
sources:
  - id: openwiki-source-6e3e162c24af43520f89dcd5
    resource: repo://docs/search-performance.md
  - id: openwiki-source-ccfe8cc4504f25c304787a46
    resource: repo://src/engine.rs
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
generated: { by: "codex", at: "2026-11-02T16:08:33.221Z" }
---

# Søgning for agenter

Hent først de ønskede kilder med `tilbud hent --alle` eller `tilbud hent --kilde tjek`. Søg derefter lokalt; der foretages ikke netværkskald ved `find`:

```sh
tilbud find "økologisk kaffe" --compact --limit 20 --offset 0
tilbud find "økologisk kaffe" --compact --limit 20 --offset 20
tilbud find kaffe --kilde tjek --butik netto --compact
tilbud find kaffe --kilde bilkatogo --include-unknown --compact
tilbud find kaffe --human
```

Brug `has_more` og næste `offset` til at hente flere sider. `total` er antallet før pagination; standardgrænsen er 20 og standardoffset er 0. En side er ikke et selvstændigt snapshot: hentning mellem sidekald eller ændret gyldighed kan ændre resultatet. `--kilde` vælger kilde og `--butik` et kæde-id. Identitet, butik og periode bevares for hver kilde; ens navn eller EAN giver ikke en fuzzy sammenfletning.

## Match og rækkefølge

Forespørgslen sænkes til små bogstaver og deles på whitespace. Alle ord skal forekomme som delstrenge i navn eller beskrivelse, og ordene kan findes i forskellige felter. Resultater med den sammenhængende frase i navnet rangeres først, derefter alle ord i navnet, derefter match som bruger beskrivelsen. Sorteringen er deterministisk med navn, kæde, kilde-id og serialiseret tilbud som efterfølgende nøgler. Katalogpriser udelades.

## Output og forbehold

JSON er standard og har `schema_version`, `command` og `data`. `data` indeholder forespørgsel, kilder og deres status, dækning og hentetid, `complete`, total, offset, limit, has_more og resultater. Fuld visning indeholder hvert kanonisk tilbud under `offer` sammen med `validity_status` og `unit_price`. `--compact` flader udvalgte felter ud, herunder kildeidentitet, butik, beskrivelse, pris, mængde, mængdegrundlag, styk, gyldighed og medlemskab; den erstatter ikke den fulde kontrakt. `--compact` og `--human` kan ikke kombineres.

Undersøg beskrivelse, medlemskab, pris, mængde og periode før sammenligning: vilkår kan være ustrukturerede eller ukendte, og enhedspris findes kun hvor den kan beregnes sikkert. Som standard medtages kun beviseligt aktuelle tilbud; `--include-unknown` medtager ukendt gyldighed, men ikke kendt fremtidige eller udløbne. Datoer vurderes i Europe/Copenhagen med inklusiv slutdato; lighed med en tidspunktgrænse er ukendt, medmindre en anden grænse afgør status. BilkaToGo oplyser ikke en tilbudsperiode, se [kildesiden](../sources/bilkatogo.md).

Kontrollér `sources`, dækning, hentetid og `complete`. Manglende snapshots springes over ved søgning på tværs af kilder; ulæselige snapshots giver delresultat og ikke-nul exitkode. En eksplicit valgt ulæselig kilde giver fejl. Friskhed og komplethed er begrænset til den enkelte kildes scope; se [Tjek](../sources/tjek.md) og [lagring](../operations/storage.md).

## Måling

[Den dokumenterede release-måling](../../docs/search-performance.md) brugte et syntetisk snapshot på 12.000 poster og 8.389.007 bytes, to opvarmninger og 30 sekventielle kald med varm filcache. Medianen var 92,353 ms før og 93,570 ms efter, p95 henholdsvis 94,335 og 95,583 ms. Målingen omfatter processtart, læsning, validering, søgning og JSON-output og er ikke en garanti for andre data eller maskiner. Der er ikke foretaget separat profilering; den observerede forskel begrunder ikke i sig selv et indeks eller en database. Udviklingshjælperen [benchmark-search.py](../../scripts/benchmark-search.py) kan gentage en tilsvarende måling med en release-binær og egne midlertidige data.
