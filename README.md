# Netcode platformer prosjekt IDATT2104 2025v
## Medlemmer
- Jonathan Jensen
- Gard Alhaug

## Introduksjon
Denne frivillige oppgaven ble gitt i Nettverksprogramming IDATT2104 i 2025 vår for de studentene som ville ha bedre karakter enn C i nettverksprogrammering-delen av faget. Oppgaven gikk ut på å lage en netcode demo, helst i et lavnivå programmeringsspråk. Vi brukte Rust og SDL for å demonstrere. Det ble anbefalt å implementere prediction, reconciliation og interpolation. 

## Implementert funksjonalitet
- Nettverksfunksjonalitet
	- Kommunikasjon via UDP. `serde` og `serde_json` blir brukt for serialisering og deserialisering av klient-server kommunikasjon.
- Spillserver
	- En server med justerbar tick rate. Støtter opp til 6 spillere, bare begrensa av antall forhåndsdefinerte farger en spiller kan ha. Dette kan utvides i `PLAYER_COLORS`-konstantet som ligger i `render.rs`.
- Spillklient
	- Prediction 
		- Spillers posisjon og fart forutsees på klientsiden med server som endelig autoritet.
	- Reconciliation
		- Spillers posisjon og fart blir samstemt med serveren basert på siste felles anerkjente spillerinput og posisjon.
	- Interpolation
		- Andre spillere blir lineært interpolert mellom posisjoner basert på serveren sin tick rate. Denne blir anslått av klienten basert på tiden siden den forrige mottatte oppdateringen.
- Spill-logikk
	- Tyngdekraft er implementert.
	- Spillere har akselerasjon istedet for å direkte sette fart. Dette gir mer naturlig bevegsele.
	- Kollisjon med plattformer.
	- Hopping og vegg-hopping er implementert.

## Mangler/Fremtidig arbeid
- Server sin tick rate bestemmer bare hvor ofte spill-state sendes ut, men spiller-logikk og fysikk bestemmes bare av hvor mange udp-pakker som sendes fra klienten til serveren. Dette gjør det mulig å speed-hacke om man senker verdien av egen `DELTA_TIME` før man kompilerer prosjektet.
- Kunne implementert luftmotstand og gameplay for å gjøre demoen litt mer interessant. En bivirkning av mangelen på luftmotstand er at man kan super-hoppe fra en vegg til en annen om man bytter retning akkurat når man treffer veggen og hopper. Dette er også en litt kjekk feature så den trenger ikke nødvendigvis å fjernes.
- Om en spiller har koblet til, fjernes aldri den spilleren fra spillet selv om den lukker vinduet sitt og slutter å sende UDP-pakker.

## Eksterne avhengigheter
SDL2 - Lavnivå C grafikkbibliotek for å vise spillet og ta spiller-input.  
`serde` - Bibliotek for å serialisere og deserialisere Rust-datastrukturer til et felles format som server og klient kan bruke.  
`serde_json` - Implementerer det spesifikke formatet som `serde`-serialiserte dataen blir omgjort til og fra.

## Installasjonsinstruksjoner
For å klone repoet kjøres:
`git clone https://github.com/Isotope-235/netcode.git`

IP og port til serveren kan konfigureres med konstantene `HOST` og `PORT` i `src/server.rs`

For å bygge binærfilen bruker man `cargo build --release`. Merk at når programmet kjøres må `assets`-mappen og SDL2-dll og -lib-filene ligge ved siden av binærfilen.

## Instruksjoner for å bruke løsningen
For å kjøre serveren brukes:
`cargo run --release -- server`

### Server kontroller
«+» — Øk tick rate  
«-» — Senk tick rate

For å kjøre en klient brukes:
`cargo run --release`

### Bevegelse
«W» — Hopp  
«A» — Venstre  
«D» — Høyre

### Netcode features (toggle)
«P» — Prediction (toggle)  
«R» — Reconciliation (toggle)  
«I» — Interpolation (toggle)  
«+» — Øk simulert ping  
«-» — Senk simulert ping

## Tester
En kan kjøre tester ved bruk av:
`cargo test`
