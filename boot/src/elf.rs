const EI_NIDENT: usize = 16;

const PT_NULL: u32 = 0;
const PT_LOAD: u32 = 1;

#[repr(C)]
struct Elf64EHdr {
    e_ident: [u8; EI_NIDENT],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

pub unsafe fn execute(binary: *const u8) -> ! {
    unsafe {
        let ehdr = binary as *const Elf64EHdr;
        let phentsize = (*ehdr).e_phentsize;
        let phnum = (*ehdr).e_phnum;

        for i in 0..phnum as usize {
            let offset = i * phentsize as usize;
            let phdr = binary.add((*ehdr).e_phoff as usize + offset) as *const Elf64Phdr;

            if (*phdr).p_type != PT_LOAD {
                continue;
            }

            let addr = (*phdr).p_vaddr as *mut u8;
            core::ptr::write_bytes(addr, 0, (*phdr).p_memsz as usize);

            if (*phdr).p_filesz != 0 {
                core::ptr::copy_nonoverlapping(
                    binary.add((*phdr).p_offset as usize),
                    addr,
                    (*phdr).p_filesz as usize,
                );

                crate::uart::printf!(
                    "Loading segment %i of size %d at %x\r\n",
                    i,
                    (*phdr).p_filesz,
                    (*phdr).p_vaddr
                );
            }
        }

        crate::uart::printf!("Jumping to kernel at 0x%x\r\n", (*ehdr).e_entry);

        core::arch::asm!(
            "jalr x0, t0, 0",
            in("t0") (*ehdr).e_entry
        );

        core::hint::unreachable_unchecked();
    }
}
