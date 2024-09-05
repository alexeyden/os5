boot.elf: target/riscv64gc-unknown-none-elf/release/boot
	cp target/riscv64gc-unknown-none-elf/release/boot boot.elf

target/riscv64gc-unknown-none-elf/release/boot:
	RUSTFLAGS="-C link-arg=-Tlink.ld" cargo build \
		  --release \
		  --target riscv64gc-unknown-none-elf --verbose
clean:
	cargo clean
	rm -f boot.elf

.PHONY: clean

target/riscv64gc-unknown-none-elf/release/boot: link.ld
-include target/riscv64gc-unknown-none-elf/release/boot.d
