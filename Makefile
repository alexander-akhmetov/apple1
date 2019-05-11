run:
	RUST_LOG=warn cargo run

apple30:
	RUST_LOG=warn cargo run -- -p roms/apple30.bin -l 280

minichess:
	RUST_LOG=error cargo run -- -p roms/ASMmchess.bin -l 300
