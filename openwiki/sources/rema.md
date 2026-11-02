---
type: integration
title: REMA 1000-kilden
description: Det offentlige produktfeed, paginering, prisvalg og kildens begrænsninger.
tags: [rema, source-adapter, pricing]
sources:
  - id: openwiki-source-0b3c9083504c39afdb61ad7c
    resource: repo://docs/verification.md
  - id: openwiki-source-836b4d70d3eac35c620705c6
    resource: repo://src/rema.rs
generated: { by: "codex", at: "2026-11-02T15:43:42.669Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T15:45:43.206Z
---

# REMA 1000-kilden

`rema` læser det offentlige produktkatalog på `https://api.digital.rema1000.dk/api/v3/products` uden login. Kataloget dokumenterer ikke tilgængelighed i en bestemt butik, og adapteren understøtter ikke butiks- eller datofilter ved hentning. `source_id` og `product_id` er produkt-id'et; kæden er `rema1000`. Butik og medlemskab er ukendte. Én entydigt oplyst stregkode bevares som EAN; flere mulige vælges ikke vilkårligt.

## Paginering og fejl

Sider hentes med 100 produkter, højst 200 sider. Adapteren kontrollerer sidenummer, sidestørrelse og totaler samt gentagne sider og produkt-id'er. Ændrede totaler, korte sider, ugyldige svar, HTTP-fejl eller sidegrænsen giver kildefejl og ufuldstændig kørsel. Ufuldstændige resultater erstatter ikke snapshotet; se [dataforløbet](../architecture/data-pipeline.md) og [lagring](../operations/storage.md).

## Prisvalg og mængder

Kun priser med begge gyldighedsgrænser, hvor hentetidspunktet ligger strengt mellem dem, kan vælges. Kildens tidsgrænser bevares som instanter. Præcis lighed med grænsen giver ikke en aktuel pris; kilden dokumenterer ikke grænsernes inklusivitet. Kampagne- eller annoncerede priser prioriteres over almindelige priser; inden for samme prioritet vælges seneste start. Modstridende beløb med samme prioritet og start giver ukendt pris og en forklaring. Ingen fremtidig eller udløbet pris anvendes som førpris; førpris sættes ukendt. Uden aktiv pris bliver posten en katalogpost med ukendt pris.

Den oprindelige `underline` bevares i beskrivelsen. Parseren accepterer en entydig indledende positiv decimal og understøttet enhed med separator eller slutning; andre former forbliver ukendte. Maksimumsantal, pris over maksimum og pant fra den valgte pris føjes til beskrivelsen, når de er angivet. Den fælles motor normaliserer mængden og beregner enhedspris, når oplysningerne rækker.

## Observeret live

Endpoint og dataformat blev ifølge [kildedokumentationen](../../docs/rema.md) verificeret 2. november 2026. En livekontrol dokumenteret i [verifikation](../../docs/verification.md) samme dag gav 3.943 rækker: 271 kampagner og 3.672 katalogposter. Det er tidsbundne observationer, ikke garantier for aktuel pris eller fuldstændig butikstilgængelighed.

Brug `cargo run -- hent --kilde rema` og derefter `cargo run -- find kirsebærsauce --kilde rema`. [Kom godt i gang](../quickstart.md) forklarer CLI, `--human` og datamappen; [Tjek-kilden](tjek.md) og [BilkaToGo](bilkatogo.md) har andre dækningsmodeller. [Offline tests](../../tests/rema.rs) dækker prisvalg og fejl.
