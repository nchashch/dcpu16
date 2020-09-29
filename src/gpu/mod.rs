extern crate nalgebra;

struct GPU {
    mat: [[f16; 4]; 4],
    mem: [f16; 0x10000]
}
