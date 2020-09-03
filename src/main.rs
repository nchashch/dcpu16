mod lib;

fn main() {
    println!("Hello, world!");
    let val = 0x01 | (0x11 << 5) | (0x07 << 10);
    let cmd = lib::Command::new(val).unwrap();
    dbg!(&cmd, cmd.code());
    let mut dcpu16 = lib::DCPU16::new();
    let mut rom: [u16; 0x10000] = [0x0000; 0x10000];
    rom[0] = (0x3f << 10) | (0x01 << 5) | 0x01;
    rom[1] = (0x22 << 10) | (0x01 << 5) | 0x02;
    rom[2] = (0x01 << 10) | (0x01 << 5) | 0x04;
    rom[3] = (0x01 << 10) | (0x00 << 5) | 0x01;
    rom[4] = (0x1f << 10) | (0x02 << 5) | 0x01;
    rom[5] = 12345;
    rom[6] = (0x02 << 10) | (0x03 << 5) | 0x01;
    dcpu16.load(rom);
    dcpu16.step();
    dbg!(dcpu16.reg);
    dcpu16.step();
    dbg!(dcpu16.reg);
    dcpu16.step();
    dbg!(dcpu16.reg);
    dcpu16.step();
    dbg!(dcpu16.reg);
    dcpu16.step();
    dbg!(dcpu16.reg);
    dcpu16.step();
    dbg!(dcpu16.reg);
}
