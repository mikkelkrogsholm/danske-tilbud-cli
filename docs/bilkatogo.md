# BilkaToGo

Kilden henter den aktuelle tilbudsavis fra BilkaToGos offentlige produktsøgning: varer, der er i sortiment og på lager i det valgte lager/butiks-scope. Standard er 1653, BilkaToGos leveringslager. Dette er hverken hele produktkataloget eller priser for alle fysiske Bilka-butikker. Adapterens `FetchOptions.store_id` kan vælge et andet numerisk scope; CLI bruger standarden.

Tilbudsavismarkeringen betyder, at varen er annonceret, men beviser ikke en rabat. Prisen og en eventuel førpris kommer kun fra det valgte scopes `storeData`; en førpris på nul behandles som ukendt. Prisernes kildeenhed er øre. Uklare indholdstekster og kildeoplyste betingelser bevares som beskrivelse. Tilbudsperioder og medlemskab antages ikke: søg med `find kaffe --kilde bilkatogo --include-unknown` for at medtage ukendt gyldighed.

Adapteren finder den offentlige søgekonfiguration i webshoppens scripts ved hver hentning og prøver højst én genindlæsning ved 401/403. Den gemmer ikke scripts eller konfiguration. Søgningen er uofficiel og kan ændre sig. Alle sider og entydige produkt-ID'er skal stemme med kildens udtømmende antal; ved fejl, manglende oplysninger, ændret sidetal eller grænseoverskridelse gemmes intet nyt snapshot. Grænserne er 100 sider à 1000 varer, 2 MB for forsiden, 6 MB pr. script, 40 MB pr. søgesvar og 30 sekunders timeout pr. HTTP-kald.

Til orientering blev [ha-middagsro's Bilka-klient](https://github.com/Dinnsen/ha-middagsro/blob/main/custom_components/middagsro/bilka.py) undersøgt; ingen kode er kopieret eller porteret derfra.
