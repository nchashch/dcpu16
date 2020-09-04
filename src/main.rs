mod dcpu;

fn main() {
    let mut dcpu16 = dcpu::DCPU16::new();
    let mut rom: [u16; 0x10000] = [0x0000; 0x10000];
    rom[0] = (0x3f << 10) | (0x01 << 5) | 0x01;
    rom[1] = (0x22 << 10) | (0x01 << 5) | 0x02;
    rom[2] = (0x01 << 10) | (0x01 << 5) | 0x04;
    rom[3] = (0x01 << 10) | (0x00 << 5) | 0x01;
    rom[4] = (0x1f << 10) | (0x02 << 5) | 0x01;
    rom[5] = 12345;
    rom[6] = (0x02 << 10) | (0x03 << 5) | 0x01;
    dcpu16.load(rom);
    loop {
        let report = dcpu16.step();
        match report {
            Ok(pc) => {
                dbg!(pc, dcpu16.reg);
            },
            Err(err) => {
                panic!(err);
            }
        }
    }
}
