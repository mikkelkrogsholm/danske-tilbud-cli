# REMA 1000

Kilden `rema` bruger det offentligt læsbare produktkatalog
`https://api.digital.rema1000.dk/api/v3/products` uden login. Det er et
produktkatalog, ikke dokumentation for tilgængelighed i en bestemt butik.
Sider hentes med 100 produkter; ændrede sidetal eller totaler, gentagne sider
eller produkt-ID'er, korte sider, ugyldige svar, HTTP-fejl og sidegrænsen på
200 gør kørslen ufuldstændig. Et ufuldstændigt resultat erstatter ikke et
snapshot. Kilden understøtter ikke butiks- eller datofilter ved hentning.

Produkt-ID er både `source_id` og `product_id`; `chain_id` er `rema1000`,
som hos Tjek. Katalogendpointet har normalt ikke stregkoder; én entydig
oplyst stregkode bevares som EAN, mens flere mulige ikke vælges vilkårligt.
Medlemskab er ukendt, og butiksdata er ikke oplyst.

Kun en pris med begge tidsgrænser, hvor hentetidspunktet ligger strengt mellem
dem, kan vælges. Kilden dokumenterer ikke, om grænserne er inklusive, så de
oprindelige tidspunkter bevares uden forlængelse; på selve grænsen er prisen
ukendt. Den fælles gyldighedskontrol giver også ukendt ved præcis lighed med
en gemt tidspunktgrænse, medmindre den anden grænse afgør gyldigheden.
Manglende eller kun fremtidige/udløbne priser giver en katalogpost med
ukendt pris. Aktive priser markeret `is_campaign` eller `is_advertised`
prioriteres over almindelige priser; inden for samme prioritet vælges seneste
start. Modstridende priser med samme prioritet og start giver ukendt pris og
en synlig forklaring. Ingen kommende normalpris bruges som førpris. Antal,
pris over maksimum og pant bevares i beskrivelsen, når kilden angiver dem.

Den oprindelige `underline` bevares. En entydig indledende mængde og enhed
(f.eks. `530 GR. / KLASSISK` eller `1.98 LTR. / 6 PK.`) læses som decimal og
normaliseres af den fælles motor; andre og omtrentlige former forbliver ukendte.
`offer_count` i snapshot og CLI er af kompatibilitetshensyn antal poster;
`promotion_count` og `catalog_count` i CLI viser fordelingen. `find` medtager
kun aktuelle kampagneposter, aldrig katalogpriser.

Test: `./scripts/test.sh`. En livekontrol kan udføres med en separat, egen
midlertidig datamappe: `tilbud kilder`, `tilbud beskriv rema`,
`tilbud hent --kilde rema` og `tilbud find kirsebærsauce --kilde rema`; tilføj
`--human` for samme oplysninger i læsbart format. Produkter, priser og
kampagner ændrer sig. Endpoint og dataformat blev verificeret 2026-11-02.
Som kilde til orientering blev [ha-middagsro's API-klient](https://github.com/Dinnsen/ha-middagsro/blob/main/custom_components/middagsro/api.py)
læst; ingen kode er kopieret, og hverken dens login eller forlængelse af
prisperioder anvendes.
