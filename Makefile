AES256_DEBUG_BIN		:=target/debug/aes-256-cbc
AES256_RELEASE_BIN		:=target/release/aes-256-cbc
AES256_BIN			:=$(AES256_DEBUG_BIN)
BIP39_DEBUG_BIN			:=target/debug/bip39
BIP39_RELEASE_BIN		:=target/release/bip39
BIP39_BIN			:=$(BIP39_DEBUG_BIN)
OBFUSKAT3_DEBUG_BIN		:=target/debug/obfuskat3
OBFUSKAT3_RELEASE_BIN		:=target/release/obfuskat3
OBFUSKAT3_BIN			:=$(OBFUSKAT3_DEBUG_BIN)
OBFUSKAT3_TARGET_PATH		:=$(shell pwd)/tmp
IPLEAK_DEBUG_BIN		:=target/debug/ipleak
IPLEAK_RELEASE_BIN		:=target/release/ipleak
IPLEAK_BIN			:=$(IPLEAK_DEBUG_BIN)
SLUGIFY_FILENAMES_DEBUG_BIN	:=target/debug/slugify-filenames
SLUGIFY_FILENAMES_RELEASE_BIN	:=target/release/slugify-filenames
SLUGIFY_FILENAMES_BIN		:=$(SLUGIFY_FILENAMES_DEBUG_BIN)
PASSWORD			:="I X@X@ Nickelback <3"
PLAINTEXT			:="Hello World"

all: fmt release

clean: cls
	rm -fr *.aes *.yaml 0b4sk8d

cls:
	@echo -e "\033[H\033[2J"

release:
	@cargo build --release
	cp target/release/slugify-filenames ~/usr/bin/
	cp target/release/aes-256-cbc ~/usr/bin/
	cp target/release/bip39 ~/usr/bin/
	cp target/release/ipleak ~/usr/bin/
	cp target/release/obfuskat3 ~/usr/bin/

fmt:
	rustfmt --edition 2021 src/*.rs
tmp:
	@rm -rf tmp
	@mkdir -p tmp/{Foo,BAR,BaZ,}/{One,TWO,THree@FouR}
	@for name in $$(find tmp -type d); do uuidgen > $$name/AA; done
	@for name in $$(find tmp -type d); do uuidgen > $$name/bB; done
	@for name in $$(find tmp -type d); do uuidgen > $$name/Cc; done
	@for name in $$(find tmp -type f); do uuidgen > $$name; done

dry-run:tmp
	@cargo run --bin slugify-filenames -- -r tmp --dry-run

test: test-slugify-filenames test-aes-256 test-obfuskat3

test-slugify-filenames: tmp cls
	@cargo run --bin slugify-filenames -- -r tmp --dry-run
	@cargo run --bin slugify-filenames -- -r tmp

test-aes-256: aes-256-key aes-256-password

test-obfuskat3: clean tmp build obfuskat3 unobfuskat3

build:
	@cargo build

silent: tmp cls
	@cargo run --bin slugify-filenames -- -r tmp --silent


coverage: cls
	grcov . --binary-path target/debug/slugify-filenames -s . -t html --branch --ignore-not-existing -o ./coverage/

aes-256-ask: cls build
	@echo $$(seq 11 | sed 's/[0-9]*/-/g' | tr -d '\n')
	@echo "$@"
	@echo $$(seq 11 | sed 's/[0-9]*/-/g' | tr -d '\n')
	@echo $(PASSWORD) | pbcopy
	@echo "\033[38;5;227mPASSWORD COPIED TO CLIPBOARD: \033[38;5;49m"$(PASSWORD)"\033[0m"
	@$(AES256_BIN) encrypt --ask-password --output-filename README.md.aes --input-filename README.md
	@$(AES256_BIN) check --ask-password --input-filename README.md.aes
	@$(AES256_BIN) decrypt --ask-password --input-filename README.md.aes --output-filename README.md
	@cargo check

aes-256-key: cls build
	@echo $$(seq 11 | sed 's/[0-9]*/-/g' | tr -d '\n')
	@echo "$@"
	@echo $$(seq 11 | sed 's/[0-9]*/-/g' | tr -d '\n')
	@$(AES256_BIN) generate --key 1000 --salt 2000 --iv 3000 --key-filename ./aes-256-key.yaml --password $(PASSWORD)
	@$(AES256_BIN) encrypt --key-filename ./aes-256-key.yaml --output-filename README.md.aes --input-filename README.md
	@$(AES256_BIN) check --key-filename ./aes-256-key.yaml --input-filename README.md.aes
	@$(AES256_BIN) decrypt --key-filename ./aes-256-key.yaml --input-filename README.md.aes --output-filename README.md
	@cargo check

aes-256-password: cls build
	@echo $$(seq 16 | sed 's/[0-9]*/-/g' | tr -d '\n')
	@echo "$@"
	@echo $$(seq 16 | sed 's/[0-9]*/-/g' | tr -d '\n')
	@$(AES256_BIN) encrypt --password $(PASSWORD) --output-filename README.md.aes --input-filename README.md
	@$(AES256_BIN) check  --password $(PASSWORD) --input-filename README.md.aes
	@$(AES256_BIN) decrypt --password $(PASSWORD) --input-filename README.md.aes --output-filename README.md
	@cargo check

aes-256: aes-256-key aes-256-password aes-256-ask

bip39: build cls
	$(BIP39_BIN)

obfuskat3: cls 0b4sk8d.yaml

0b4sk8d.yaml: $(OBFUSKAT3_BIN)
	$(OBFUSKAT3_BIN) from $(OBFUSKAT3_TARGET_PATH)

unobfuskat3:
	$(OBFUSKAT3_BIN) undo 0b4sk8d.yaml

ipleak: cls build
	$(IPLEAK_BIN)

load: clean build
	./aestest.sh


$(AES256_RELEASE_BIN):
	@cargo build --release

$(AES256_DEBUG_BIN):
	@cargo build



.PHONY: all release fmt tmp test dry-run coverage aes256 build clean test-e2e test-aes-256 test-slugify-filenames bip39 ipleak obfuskat3
