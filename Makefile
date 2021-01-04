 
.PHONY: release, test, dev

release:
	cargo update
	cargo build --release
	strip target/release/source_microservice

build:
	cargo build

dev:
	# . ./ENV.sh; backper
	cargo run;

test:
	cargo test