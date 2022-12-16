use simd_string::*;

fn main() {
    let s = "                          123456";

    let s = s.simd_parse::<u64>();

    println!("{:?}", s);
}
