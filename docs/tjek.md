# Tjek-kilden

`TjekAdapter::new()` vælger alle 14 kæder; `select_chains(&["rema1000"])`
vælger et udsnit og afviser ukendte ID'er. Hentningen er en biblioteksfunktion,
ikke en CLI-kommando. `engine::run` returnerer både normaliserede tilbud,
afviste poster, kildefejl og dækning.

Kilden er `https://squid-api.tjek.com/v2`. Adapteren henter `/offers` med
`dealer_id`, `limit=100` og `offset`, til feedet er udtømt. Hvis Tjek svarer
`PAGINATION_MAX_REACHED`, henter adapteren i stedet alle returnerede
`/catalogs` for kæden og derefter `/offers` med både `dealer_id` og
`catalog_id`. Den vej dækker de hentede kataloger; fritstående tilbud efter
den direkte grænse er ikke omfattet. Dækningen angiver altid den anvendte vej.
En tom liste giver ukendt tilbudsdækning og en kildefejl. Kæde og dansk land
kontrolleres på hver post, og i katalogvejen også katalogtilhørsforhold.
Butiks-ID bevares fra tilbud eller katalog; et butiksfilter inkluderer
landsdækkende tilbud og den valgte butiks lokale tilbud. Datofilter anvendes
på hentede poster som overlap med den valgte kalenderdag i Europe/Copenhagen;
det er ikke en historisk søgning i kilden. Ukendt gyldighed ved datofilter giver
fejl. Sider hentes sekventielt, højst 200 pr. liste, og gentagne sider eller
fejl giver delvis dækning. HTTP har 10 sekunders timeout, højst tre forsøg og
respekterer Retry-After; en forsinkelse over fem sekunder giver synlig fejl
frem for et for tidligt genforsøg.

Priser læses fra JSON-tal uden binære flydende kommatal. `pre_price` er en
kildesammenligning, ikke nødvendigvis en tidligere normalpris; beskrivelsen
bevares. F.eks. kan den være prisen over en købsgrænse. Tjeks `pieces.from/to`
bruges som antal og `size.from/to` som størrelse pr. del, når begge er oplyst;
`pieces.max` er ikke et pakkeantal. Oplyste `pieces.min/max` bevares i
beskrivelsen. En live-post for Burgerboost angav `2x175 g`, størrelse 175 g,
antal 2 og max 6. Medlemskab er ukendt, da feedet ikke giver en verificeret
struktureret medlemsmarkering. Følgende kæde-ID'er er stabile internt:

Den fulde hybridkørsel 2. november 2026 ca. 14:44 UTC gav 5.568
normaliserede tilbud, 0 dubletter, 0 afviste poster og 0 kildefejl. De tre
føtex-poster uden overskrift beholdes med ukendt navn. Antallene nedenfor er
hentede poster inden normalisering; Bilkas katalogdækning har den ovenfor
beskrevne begrænsning.

| Kæde-ID | Dealer-ID | Poster | Hentning |
| --- | --- | ---: | --- |
| rema1000 | 11deC | 113 | Direkte |
| netto | 9ba51 | 247 | Direkte |
| lidl | 71c90 | 559 | Direkte |
| foetex | bdf5A | 941 | Direkte |
| bilka | 93f13 | 1861 | Kataloger |
| 365discount | DWZE1w | 149 | Direkte |
| meny | 267e1m | 186 | Direkte |
| superbrugsen | 0b1e8 | 254 | Direkte |
| kvickly | c1edq | 469 | Direkte |
| brugsen | d311fg | 77 | Direkte |
| spar | 88ddE | 91 | Direkte |
| loevbjerg | 65caN | 420 | Direkte |
| min-koebmand | 603dfL | 77 | Direkte |
| abc-lavpris | 70d42L | 124 | Direkte |

En separat stikprøve 2. november 2026 kl. 14:38 UTC på
`/offers?dealer_id=…&limit=1` gav HTTP 200 og én korrekt DK/DKK-post for hver
af de 14 kæder. Det er en stikprøve og ikke bevis for fuld dækning. Direkte
Bilka-pagination stopper ved offset 1000 med `PAGINATION_MAX_REACHED`, hvilket
er grunden til katalogfallback. Antal er observationer, ikke garantier.

Gentag en fuld live-kørsel med `cargo run --example tjek_smoke` eller kun REMA
med `cargo run --example tjek_smoke -- rema1000`; brug en egen midlertidig
`CARGO_TARGET_DIR`, hvis build-output skal fjernes efterfølgende. Programmet
returnerer en fejlkode ved kildefejl og viser afvisningsårsager.

Referencer til udforskning: [tilbudstrolden-mcp](https://github.com/olgasafonova/tilbudstrolden-mcp),
[kundeavis-mcp](https://github.com/donadelicc/kundeavis-mcp) og
[ha-middagsro chains.py](https://github.com/Dinnsen/ha-middagsro/blob/main/custom_components/middagsro/chains.py).
Ingen kode er kopieret eller porteret fra disse projekter; dealer-ID'er er
verificeret mod live-responserne.
