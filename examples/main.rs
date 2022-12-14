use simd_string::*;

fn main() {
    let s = "                          123456";

    let s = s.to_simd_string();
    let s = s.parse::<u64>();

    println!("{:?}", s);
}
