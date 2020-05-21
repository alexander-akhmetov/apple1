TAG ?= latest

.PHONY: all

run:
	RUST_LOG=warn cargo run

docker/build:
	docker build -t akhmetov/apple1:$(TAG) .

docker/run:
	docker run -it akhmetov/apple1:$(TAG)

apple30:
	RUST_LOG=warn cargo run -- -p roms/apple30.bin -a 280

minichess:
	RUST_LOG=error cargo run -- -p roms/ASMmchess.bin -a 300
