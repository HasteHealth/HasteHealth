fn main() {
    // `sqlx::migrate!` embeds these, so a new migration must rebuild the crate.
    println!("cargo::rerun-if-changed=pg-migrations");
}
