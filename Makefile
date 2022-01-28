TAG ?= latest

.PHONY: all

run:
	RUST_LOG=warn cargo run --features binary

docker/build:
	docker build -t akhmetov/apple1:$(TAG) .

docker/run:
	docker run -it akhmetov/apple1:$(TAG)

apple30:
	RUST_LOG=warn cargo run --features binary -- -p roms/apple30.bin -a 280

minichess:
	RUST_LOG=error cargo run --features binary -- -p roms/ASMmchess.bin -a 300
