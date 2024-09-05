FLASH_DEV?=/dev/sda

boot.img: boot/boot.elf
	riscv64-elf-objcopy -O binary boot/boot.elf boot.bin
	scripts/gencksum boot.bin boot.img

boot/boot.elf:
	make -C boot

clean:
	rm -f boot.img boot.bin boot.img.S boot.elf.S
	make -C boot clean

flash: boot.img
	[ -b $(FLASH_DEV) ]
	sudo dd if=boot.img of=$(FLASH_DEV) bs=8192 seek=1
	sync

boot.img.S: boot.img
	riscv64-elf-objdump -m riscv:rv64 -b binary -D boot.img > boot.img.S

boot.elf.S: boot/boot.elf
	riscv64-elf-objdump -S boot/boot.elf > boot.elf.S

.PHONY: clean boot/boot.elf
