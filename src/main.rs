mod lib;

fn main() {
    println!("Hello, world!");
    let val = 0x01 | (0x11 << 5) | (0x07 << 10);
    let cmd = lib::Command::new(val).unwrap();
    dbg!(&cmd, cmd.code());
}
