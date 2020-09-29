mod dcpu;
mod assembly;

fn main() {

    let mut dcpu16 = dcpu::DCPU16::new();
    let mut rom: [u16; 0x10000] = [0x0000; 0x10000];
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut arr_addr = 1000;
    rom[arr_addr] = arr.len() as u16;
    for (i, x) in arr.iter().enumerate() {
        rom[arr_addr + 1 + i] = *x;
    }
    let mut pc = 0;

    let cmds = [
        cmd(0x01, 0x01, 0x1f), // SET B, 1000
        (arr_addr + 1) as u16,
        cmd(0x01, 0x02, 0x1e), // SET C, [0x50]
        arr_addr as u16,
        cmd(0x02, 0x02, 0x01), // ADD, C, B
        cmd(0x02, 0x00, 0x09), // ADD A, [B]
        cmd(0x02, 0x01, 0x1f), // ADD B, 1
        1,
        cmd(0x13, 0x01, 0x02), // IFN B, C
        cmd(0x00, 0x01, 0x1f), // JSR 0x04
        4
    ];
    let source_text =
"
set b, 1001;
set c, [1000];
add c, b;
add a, [b];
add b, 1;
ifl b, c;
jsr 5;
";
    let program = assembly::parse(source_text).unwrap();
    // for command in program.iter() {
    //     dbg!(command);
    // }
    let code = assembly::generate_code(program);

    for (i, word) in code.iter().enumerate() {
        dbg!(word, cmds[i]);
        rom[i] = *word;
    }

    // for (i, cmd) in cmds.iter().enumerate() {
    //     dbg!(cmd);
    //     rom[i] = *cmd;
    // }

    dcpu16.load(rom);
    loop {
        let report = dcpu16.step();
        match report {
            Ok(pc) => {
                dbg!(pc, dcpu16.reg);
            },
            Err(_) => {
                break;
            }
        }
    }
}

fn cmd(op: u16, b: u16, a: u16) -> u16 {
    (a << 10) | (b << 5) | op
}
