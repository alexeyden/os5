use crate::boot_panic;

#[derive(Default)]
struct Crc16(u16);

impl Crc16 {
    /* crctab calculated by Mark G. Mendel, Network Systems Corporation */
    const TAB: [u16; 256] = [
        0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50a5, 0x60c6, 0x70e7, 0x8108, 0x9129, 0xa14a,
        0xb16b, 0xc18c, 0xd1ad, 0xe1ce, 0xf1ef, 0x1231, 0x0210, 0x3273, 0x2252, 0x52b5, 0x4294,
        0x72f7, 0x62d6, 0x9339, 0x8318, 0xb37b, 0xa35a, 0xd3bd, 0xc39c, 0xf3ff, 0xe3de, 0x2462,
        0x3443, 0x0420, 0x1401, 0x64e6, 0x74c7, 0x44a4, 0x5485, 0xa56a, 0xb54b, 0x8528, 0x9509,
        0xe5ee, 0xf5cf, 0xc5ac, 0xd58d, 0x3653, 0x2672, 0x1611, 0x0630, 0x76d7, 0x66f6, 0x5695,
        0x46b4, 0xb75b, 0xa77a, 0x9719, 0x8738, 0xf7df, 0xe7fe, 0xd79d, 0xc7bc, 0x48c4, 0x58e5,
        0x6886, 0x78a7, 0x0840, 0x1861, 0x2802, 0x3823, 0xc9cc, 0xd9ed, 0xe98e, 0xf9af, 0x8948,
        0x9969, 0xa90a, 0xb92b, 0x5af5, 0x4ad4, 0x7ab7, 0x6a96, 0x1a71, 0x0a50, 0x3a33, 0x2a12,
        0xdbfd, 0xcbdc, 0xfbbf, 0xeb9e, 0x9b79, 0x8b58, 0xbb3b, 0xab1a, 0x6ca6, 0x7c87, 0x4ce4,
        0x5cc5, 0x2c22, 0x3c03, 0x0c60, 0x1c41, 0xedae, 0xfd8f, 0xcdec, 0xddcd, 0xad2a, 0xbd0b,
        0x8d68, 0x9d49, 0x7e97, 0x6eb6, 0x5ed5, 0x4ef4, 0x3e13, 0x2e32, 0x1e51, 0x0e70, 0xff9f,
        0xefbe, 0xdfdd, 0xcffc, 0xbf1b, 0xaf3a, 0x9f59, 0x8f78, 0x9188, 0x81a9, 0xb1ca, 0xa1eb,
        0xd10c, 0xc12d, 0xf14e, 0xe16f, 0x1080, 0x00a1, 0x30c2, 0x20e3, 0x5004, 0x4025, 0x7046,
        0x6067, 0x83b9, 0x9398, 0xa3fb, 0xb3da, 0xc33d, 0xd31c, 0xe37f, 0xf35e, 0x02b1, 0x1290,
        0x22f3, 0x32d2, 0x4235, 0x5214, 0x6277, 0x7256, 0xb5ea, 0xa5cb, 0x95a8, 0x8589, 0xf56e,
        0xe54f, 0xd52c, 0xc50d, 0x34e2, 0x24c3, 0x14a0, 0x0481, 0x7466, 0x6447, 0x5424, 0x4405,
        0xa7db, 0xb7fa, 0x8799, 0x97b8, 0xe75f, 0xf77e, 0xc71d, 0xd73c, 0x26d3, 0x36f2, 0x0691,
        0x16b0, 0x6657, 0x7676, 0x4615, 0x5634, 0xd94c, 0xc96d, 0xf90e, 0xe92f, 0x99c8, 0x89e9,
        0xb98a, 0xa9ab, 0x5844, 0x4865, 0x7806, 0x6827, 0x18c0, 0x08e1, 0x3882, 0x28a3, 0xcb7d,
        0xdb5c, 0xeb3f, 0xfb1e, 0x8bf9, 0x9bd8, 0xabbb, 0xbb9a, 0x4a75, 0x5a54, 0x6a37, 0x7a16,
        0x0af1, 0x1ad0, 0x2ab3, 0x3a92, 0xfd2e, 0xed0f, 0xdd6c, 0xcd4d, 0xbdaa, 0xad8b, 0x9de8,
        0x8dc9, 0x7c26, 0x6c07, 0x5c64, 0x4c45, 0x3ca2, 0x2c83, 0x1ce0, 0x0cc1, 0xef1f, 0xff3e,
        0xcf5d, 0xdf7c, 0xaf9b, 0xbfba, 0x8fd9, 0x9ff8, 0x6e17, 0x7e36, 0x4e55, 0x5e74, 0x2e93,
        0x3eb2, 0x0ed1, 0x1ef0,
    ];

    pub fn update(&mut self, b: u8) {
        /*
         * updcrc macro derived from article Copyright (C) 1986 Stephen Satchell.
         *  NOTE: First srgument must be in range 0 to 255.
         *        Second argument is referenced twice.
         *
         * Programmers may incorporate any or all code into their programs,
         * giving proper credit within the source. Publication of the
         * source routines is permitted so long as proper credit is given
         * to Stephen Satchell, Satchell Evaluations and Chuck Forsberg,
         * Omen Technology.
         */
        self.0 = unsafe { Self::TAB.get_unchecked(((self.0 >> 8) & 255) as usize) }
            ^ (self.0 << 8)
            ^ b as u16
    }

    pub fn finish(&mut self) -> u16 {
        self.update(0);
        self.update(0);
        self.0
    }
}

const XON: u8 = 0x11;
const XOFF: u8 = 0x13;
const XONESC: u8 = 0x11 | 0x80;
const XOFFESC: u8 = 0x13 | 0x80;
const ZDLE: u8 = 0x18;
const ZCRCE: u8 = b'h';
const ZCRCG: u8 = b'i';
const ZCRCQ: u8 = b'j';
const ZCRCW: u8 = b'k';
const ZBIN: u8 = b'A';
const ZHEX: u8 = b'B';

const ZRQINIT: u8 = 0;
const ZRINIT: u8 = 1;
const ZACK: u8 = 3;
const ZFILE: u8 = 4;
const ZFIN: u8 = 8;
const ZRPOS: u8 = 9;
const ZDATA: u8 = 10;
const ZEOF: u8 = 11;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Sym {
    Chr(u8),
    Esc(u8),
}

impl Sym {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Chr(v) => v,
            Self::Esc(v) => v,
        }
    }
}

#[derive(Clone, Copy)]
struct Header {
    typ: u8,
    data: [u8; 4],
}

struct Subpacket<'a> {
    typ: u8,
    data: &'a [u8],
}

pub struct ZModem {
    rx: fn() -> u8,
    tx: fn(u8),
}

impl ZModem {
    pub fn new(rx: fn() -> u8, tx: fn(u8)) -> Self {
        Self { rx, tx }
    }

    fn rx_ascii(&self) -> u8 {
        loop {
            match (self.rx)() {
                XON | XOFF => continue,
                c => return c,
            }
        }
    }

    fn rx_bin(&self) -> Sym {
        loop {
            match (self.rx)() {
                ZDLE => break,
                XON | XOFF | XONESC | XOFFESC => continue,
                c => return Sym::Chr(c),
            }
        }

        loop {
            match (self.rx)() {
                c @ ZCRCE | c @ ZCRCG | c @ ZCRCQ | c @ ZCRCW => return Sym::Esc(c),
                XON | XOFF | XONESC | XOFFESC => continue,
                c => return Sym::Chr(c ^ 0x40),
            }
        }
    }

    fn tx_bin(&self, data: &[u8]) {
        for b in data {
            (self.tx)(*b);
        }
    }

    fn tx_hex(&self, data: &[u8]) {
        fn enc_nibble(b: u8) -> u8 {
            if b >= 0xa { b'a' + (b - 0xa) } else { b'0' + b }
        }

        for b in data {
            (self.tx)(enc_nibble(b >> 4));
            (self.tx)(enc_nibble(b & 0xf));
        }
    }

    fn rx_hex_byte(&self) -> u8 {
        fn dec_nibble(b: u8) -> u8 {
            if b >= b'0' && b <= b'9' {
                b - b'0'
            } else {
                0xa + b - b'a'
            }
        }

        (dec_nibble(self.rx_ascii()) << 4) | dec_nibble(self.rx_ascii())
    }

    fn rx_header(&self) -> Header {
        while self.rx_ascii() != b'*' {}
        while self.rx_ascii() != ZDLE {}

        match self.rx_ascii() {
            ZHEX => self.rx_hex_header(),
            ZBIN => self.rx_bin16_header(),
            c => boot_panic!("unexpected header type: %d", c),
        }
    }

    fn rx_hex_header(&self) -> Header {
        let mut buf = [0u8; 7];
        let mut crc = Crc16::default();

        for c in &mut buf {
            *c = self.rx_hex_byte();
            crc.update(*c);
        }
        let crc = crc.finish();

        if crc != 0 {
            boot_panic!(
                "invalid hex header CRC: %x %x %x %x %x",
                unsafe { buf.get_unchecked(0) },
                unsafe { buf.get_unchecked(1) },
                unsafe { buf.get_unchecked(2) },
                unsafe { buf.get_unchecked(3) },
                unsafe { buf.get_unchecked(4) }
            );
        }

        if self.rx_ascii() == b'\r' {
            self.rx_ascii(); // LF
        }

        Header {
            typ: unsafe { *buf.get_unchecked(0) },
            data: unsafe { buf.get_unchecked(1..5).try_into().unwrap_unchecked() },
        }
    }

    fn rx_bin16_header(&self) -> Header {
        let mut buf = [0u8; 7];
        let mut crc = Crc16::default();

        for c in &mut buf {
            *c = self.rx_bin().as_u8();
            crc.update(*c);
        }
        let crc = crc.finish();

        if crc != 0 {
            boot_panic!(
                "invalid BIN16 header CRC: %x %x %x %x %x",
                unsafe { buf.get_unchecked(0) },
                unsafe { buf.get_unchecked(1) },
                unsafe { buf.get_unchecked(2) },
                unsafe { buf.get_unchecked(3) },
                unsafe { buf.get_unchecked(4) }
            );
        }

        Header {
            typ: unsafe { *buf.get_unchecked(0) },
            data: unsafe { buf.get_unchecked(1..5).try_into().unwrap_unchecked() },
        }
    }

    fn tx_hex_header(&self, typ: u8, data: [u8; 4]) {
        let mut crc = Crc16::default();
        crc.update(typ);
        for b in data {
            crc.update(b);
        }
        let crc = crc.finish();

        (self.tx)(b'*');
        (self.tx)(b'*');
        (self.tx)(ZDLE);
        (self.tx)(b'B');
        self.tx_hex(&[typ]);
        self.tx_hex(&data);
        self.tx_hex(&crc.to_be_bytes());
        self.tx_bin(b"\r\n\x11");
    }

    fn rx_subpacket<'a>(&self, data: &'a mut [u8]) -> Subpacket<'a> {
        let mut len = 0;
        let mut crc = Crc16::default();
        let mut typ = 0;

        for b in &mut *data {
            match self.rx_bin() {
                Sym::Esc(c) => {
                    crc.update(c);
                    typ = c;
                    break;
                }
                Sym::Chr(c) => {
                    crc.update(c);
                    *b = c;
                    len += 1;
                }
            }
        }

        if ![ZCRCE, ZCRCG, ZCRCQ, ZCRCW].contains(&typ) {
            boot_panic!("invalid subpacket type: 0x%x", typ);
        }

        crc.update(self.rx_bin().as_u8());
        crc.update(self.rx_bin().as_u8());

        if crc.finish() != 0 {
            boot_panic!("invalid subpacket CRC (subpacket 0x%x)", typ);
        }

        Subpacket {
            typ,
            data: unsafe { data.get_unchecked(..len) },
        }
    }

    pub fn recv_file(self, buffer: &mut [u8]) -> usize {
        crate::uart::printf!("Receiving boot image via ZMODEM...\r\n");

        let header = self.rx_header();
        if header.typ != ZRQINIT {
            boot_panic!(
                "unexpected response header: %d (expected ZRQINIT)",
                header.typ
            );
        }

        self.tx_hex_header(ZRINIT, [0, 0, 0, 0]);

        let zfile = self.rx_header();
        if zfile.typ != ZFILE {
            boot_panic!("unexpected response: %d (expected ZFILE)", header.typ);
        }

        let mut data = [0u8; 64];
        let subpacket = self.rx_subpacket(&mut data);

        let filename = {
            let len = subpacket.data.iter().position(|&c| c == 0).unwrap_or(0);
            unsafe { data.get_unchecked(..len) }
        };

        self.tx_hex_header(ZRPOS, [0, 0, 0, 0]);

        let header = self.rx_header();
        if header.typ != ZDATA {
            boot_panic!("unexpected header: %d (expected ZDATA)", header.typ);
        }

        let mut offset = 0usize;

        loop {
            let packet = self.rx_subpacket(unsafe { buffer.get_unchecked_mut(offset..) });

            offset += packet.data.len();

            if packet.typ == ZCRCE {
                break;
            } else if packet.typ == ZCRCG {
                continue;
            } else {
                boot_panic!("unsupported subpacket type: 0x%x", packet.typ);
            }
        }

        let header = self.rx_header();
        if header.typ != ZEOF {
            boot_panic!("unexpected header: %d (expected ZEOF)", header.typ);
        }

        self.tx_hex_header(ZRINIT, [0, 0, 0, 0]);

        let header = self.rx_header();
        if header.typ != ZFIN {
            boot_panic!("unexpected header: %d (expected ZFIN)", header.typ);
        }

        self.tx_hex_header(ZFIN, [0; 4]);

        crate::uart::printf!("\r\nFile name: %s\r\n", filename.as_ptr());

        return offset;
    }
}
