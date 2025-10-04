.PHONY: all build install local-install clean

PREFIX ?= /usr/local
MANDIR = $(PREFIX)/share/man/man1

all: build

build:
	cargo build --release

install:
	cargo install --path . --root $(DESTDIR)$(PREFIX) --no-track
	install -d $(DESTDIR)$(MANDIR)
	gzip -c geo-loc.1 > geo-loc.1.gz
	install -m 644 geo-loc.1.gz $(DESTDIR)$(MANDIR)/geo-loc.1.gz
	rm geo-loc.1.gz

local-install: build
	mkdir -p $(HOME)/bin
	install -m 755 target/release/geo-loc $(HOME)/bin/geo-loc
	mkdir -p $(HOME)/man/man1
	cp geo-loc.1 $(HOME)/man/man1/geo-loc.1

clean:
	cargo clean