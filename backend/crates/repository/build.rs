fn main() {
    println!("cargo::rerun-if-changed=pg-migrations");
    println!("cargo::rerun-if-changed=src");
}
