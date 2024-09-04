FLASH_DEV?=/dev/sda

boot.img: boot.elf
	riscv64-elf-objcopy -O binary boot.elf boot.bin
	scripts/gencksum boot.bin boot.img
	rm -f boot.bin

boot.elf: boot.S
	riscv64-elf-as boot.S -o boot.elf

clean:
	rm -f boot.elf boot.img boot.img.S boot.elf.S

flash: boot.img
	[ -b $(FLASH_DEV) ]
	sudo dd if=boot.img of=$(FLASH_DEV) bs=8192 seek=1
	sync

boot.img.S: boot.img
	riscv64-elf-objdump -m riscv:rv64 -b binary -D boot.img > boot.img.S

boot.elf.S: boot.elf
	riscv64-elf-objdump -d boot.elf > boot.elf.S

.PHONY: clean
