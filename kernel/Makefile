kernel.elf: target/riscv64gc-unknown-none-elf/release/kernel
	cp target/riscv64gc-unknown-none-elf/release/kernel kernel.elf

target/riscv64gc-unknown-none-elf/release/kernel:
	RUSTFLAGS="-C link-arg=-Tlink.ld" cargo build \
		  --release \
		  --target riscv64gc-unknown-none-elf --verbose
clean:
	cargo clean
	rm -f kernel.elf

.PHONY: clean

target/riscv64gc-unknown-none-elf/release/kernel: link.ld
-include target/riscv64gc-unknown-none-elf/release/kernel.d
