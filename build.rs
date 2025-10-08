fn main() {
    println!("cargo:rustc-link-arg=-T.cargo/commands.ld");
}
