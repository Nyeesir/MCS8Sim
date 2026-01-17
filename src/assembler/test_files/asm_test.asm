ORG 800H
LXI H,PIERWLICZB ;wczytanie adresu pierwszej wiadomosci do HL
RST 3 ;wyswietlenie wiadomosci
CALL WPROWADZLICZB ;procedura wpisywania liczby
STA LICZBZMIENNA ;zapisujemy ja tymczasowo w pamieci
LXI H,DRUGALICZB ;;wczytanie adresu drugiej wiadomosci do HL
RST 3 ;wyswietlenie wiadomosci
CALL WPROWADZLICZB ;procedura wpisywania liczby
LDA LICZBZMIENNA ;wczytujemy pierwsza liczbe z pamieci
MOV E,A ;ustawiamy jak jako mnoznik
MOV L,B  ;ustawiamy druga liczbe jako mnoznik
CALL MNOZENIEINIC ;mnozymy je
MOV B,H
MOV C,L
LXI H,WYNIKMNOZENIA ;ladujemy komunikat o wyniku mnozenia
RST 3

LXI H,BINSTRG ;zapisuje wynik mnozenia do ramu
MOV M,C
INX H
MOV M,B

MVI D,16 ;licznik glownej petli
MAINLOOP:

MVI B,5 ;licznik petli
LXI H,BCDSTRG ;ladujemy adres najmlodszego bajtu bcd

LOOPBCDMULT: ;petla odpowiedzialna za podwajanie wartosci kolejnych bajtow wynikowych
MOV A,M
ADD M ;podwaja wartosc bajtu
MOV M,A
INX H ;przechodzi do starszego bajtu
DCR B
JNZ LOOPBCDMULT

MVI C,0 ;zerujemy rejestr z przeniesieniami
LXI H,BINSTRG ;ladujemy adres bajtu bin
MOV A,D
CPI 9 ;jezeli jestesmy w pierwszych 8 iteracjkach petli to korzysta ze starszego bajtu bin
JM NIEPRZECH
INX H
NIEPRZECH:
MOV A,M
RAL ;przesuwamy go w lewo
JNC SKIPSETC
MVI C,1 ;jezeli wygenerowane jest przeniesienie to zapisujemy je w rejestrze pomocniczym
SKIPSETC:
MOV M,A

MVI B,5 ;licznik petli
LXI H,BCDSTRG ;ladujemy adres najmlodszego bajtu bcd

LOOPBCDCARRY: ;petla ktora generuje przeniesienia do starszego bajtu jezeli wartosc mlodszego jest wieksza od 9
MOV A,M
ADD C ;dodaje bit przeniesienia
MVI C,0 ;zeruje przeniesienie
CPI 10 ;sprawdza czy wartosc jest wieksza od 9
JM SKIPCARRY
SUI 10 ;jezeli tak to odejmuje 10 
MVI C,1 ;i ustawia przeniesienie do wykorzystania na wyzszym bajcie
SKIPCARRY:
MOV M,A
INX H ;przechodzi na starszy bajt
DCR B
JNZ LOOPBCDCARRY

DCR D 
JNZ MAINLOOP

MVI B,4 
LXI H,BCDSTRG ;wczytuje adres najstarszego bajtu
LOOPINXH: ;przechodzi do najmlodszego bajtu
INX H
DCR B
JNZ LOOPINXH

MVI B,5
PRTLOOP: ;wyswietla bajty od najmlodszego do najstarszeggo
MOV A,M
ADI 30H ;dodaje do kazdego bajtu 30H zeby wynik wyszedl w ascii
RST 1
DCX H
DCR B
JNZ PRTLOOP

MVI B,2 ;czyszczenie na wszelki wypadek pamieci zarezerwowanej na bin
LXI H,BINSTRG
CLEARLOOPBIN:
MVI M,0
INX H
DCR B
JNZ CLEARLOOPBIN

MVI B,5 ;czyszczenie pamieci zarezerwowanej na bcd
LXI H,BCDSTRG
CLEARLOOPBCD:
MVI M,0
INX H
DCR B
JNZ CLEARLOOPBCD


HLT ;koniec programu



WPROWADZLICZB:
;na poczatku planujemy skladowac pojedyncze cyfry liczby w rejestrach B,C,D
;B - setki, C - dziesiatki, D - jednosci

RST 2 ;wczytanie pierszej liczby
CALL SPRAWDZCYFRPIERW ;sprawdzanie czy znak lezy w odp zakresie
MOV B,A

RST 2 ;wczytanie drugiej liczby
CALL SPRAWDZCYFR ;sprawdzanie czy znak lezy w odp zakresie
MOV C,A
CPI 0DH ;sprawdzamy czy wpisany znak to enter
JZ OBSLJEDNCYFR ;obsluga wpisywania liczby jednocyfrowej

RST 2 ;wczytanie trzeciej liczby
CALL SPRAWDZCYFR ;sprawdzanie czy znak lezy w odp zakresie
MOV D,A
CPI 0DH ;sprawdzamy czy wpisany znak to enter
JZ OBSLDWCYFR ;obsluga wpisywania liczby dwucyfrowej
JMP SPRWADZCZYWZAKR ;po wpisaniu calej liczby przechodzily do funkcji ktora sprawdza czy wartosc miesci sie w jednym bajcie

SPRAWDZCYFR:
CPI 0DH ;sprawdzanie czy podany znak jest enterem
RZ
SPRAWDZCYFRPIERW:
CPI 30H ;sprawdzanie czy podany znak ma wieksza wartosc niz 0
JM NIEPOPRCYFR
CPI 3AH ;sprawdzanie czy podany znak ma wieksza wartosc niz 9
JP NIEPOPRCYFR
SUI 30H ;zamiana cyfry z ascii na binarne
RET

OBSLJEDNCYFR:
MOV D,B ;przesuwa wartosc z rejestru setek do jednosci
MVI B,0 ;zeruje pozostale rejestry
MVI C,0
JMP SPRWADZCZYWZAKR

OBSLDWCYFR: 	MOV D,C ;przesuwa wartosc z rejestru dziesiatek do jednosci
MOV C,B ;przesuwa wartosc z rejestru setek do dziesiatek
MVI B,0 ;zeruje rejestr setek

SPRWADZCZYWZAKR:
MOV A,B 
CPI 3 ;sprawdzamy czy liczba setek jest rowna lub wieksza 3
JP NIEPOPRLICZB
CPI 2 ;sprawdzamy czy liczba setek jest mniejsza 2, jezeli tak to pomijamy reszte sprawdzan
JNZ SKIPBLOKSPRAWDZ
MOV A,C
CPI 6 ;wiemy ze liczba setek wynosi 2, sprawdzamy czy liczba dziesiatek jest mniejsza od 6
JP NIEPOPRLICZB
CPI 5 ;wiemy ze liczba setek wynosi 2, sprawdzamy czy liczba dziesiatek jest mniejsza od 5 i jak tak to pomijamy 
JNZ SKIPBLOKSPRAWDZ
MOV A,D
CPI 6 ;wiemy ze pozostale cyfry to sa w odp zakresie, sprawdzamy czy liczba dziesiatek jest mniejsza od 6
JP NIEPOPRLICZB

SKIPBLOKSPRAWDZ:
;zamiana ilosci dziesiatek na prawdziwo wartosc 
MOV A,C ;jezeli liczba dziesiatek to 0 to pomijamy ten blok
CPI 0
JZ SKIPMNOZ10
MVI E,10 ;ustawiamy mnoznik na 10
MOV L,C ;ustawiamy C jako mnozna
CALL MNOZENIEINIC
MOV C,L ;wynik wraca do C
SKIPMNOZ10:


MOV A,B 
CPI 0 ;jezeli liczba stek to 0 to pomijamy ten blok
JZ SKIPMNOZ100
;zamiana ilosci setek na prawdziwo wartosc 
MVI E,100 ;ustawiamy mnoznik na 100
MOV L,B ;ustawiamy B jako mnozna
CALL MNOZENIEINIC
MOV B,L ;wynik wraca do B
SKIPMNOZ100:

;laczymy wszystkie rejestry w jeden
MOV A,B
ADD C
ADD D
MOV B,A
RET ;wracamy na gore programu


MNOZENIEINIC: ;inicjujemy mnozenie, czyscimy potrzebne rejestry
MVI H,0
MVI A,0
MNOZENIE: ;mnozy E razy wartosc L, wynik zapisuje w HL
ADD L ;dodajemy do akumulatora caly czas wartosc l
JNC SKIP2 ;jezeli nie ma bitu przeniesienia to pomijamy nastepny krok
INR H ;jezeli wygenerujemy bit przeniesienia to dodajemy go do rejestru H
SKIP2:
DCR E ;zminiejszamy licznik petli
JNZ MNOZENIE ;jezeli licznik wiekszy od 0 to wracamy na poczatek petli
MOV L,A
RET

NIEPOPRCYFR:
INX SP ;czyscimy syf po callu do ktorego nie wrocilismy
INX SP
JP NIEPOPRLICZB

NIEPOPRLICZB:
LXI H,WPROWADZPON ;ladujemy komunikat 
RST 3 ;wyswietlamy go
MVI H,0 ;czyscimiy H
JMP WPROWADZLICZB ;ponowna proba wpisania liczby


PIERWLICZB: DB 'Podaj pierwsza liczbe: @'
WPROWADZPON: DB 10,13,'Niepoprawna liczba, sprobuj ponownie: @'
DRUGALICZB: DB 10,13,'Podaj druga liczbe: @'
WYNIKMNOZENIA: DB 10,13,'Wynik mnozenia: @'
LICZBZMIENNA: DB 0
BCDSTRG: 	 DB 0,0,0,0,0
BINSTRG: 	 DB 0,0