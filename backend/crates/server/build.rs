fn main() {
    println!("cargo::rerun-if-changed=css");
    println!("cargo::rerun-if-changed=src");
    let mut tailwindcss_process = std::process::Command::new("./tools/tailwindcss")
        .args(["-i", "./css/app.css"])
        .args(["-o", "./public/css/app.css"])
        .spawn()
        .expect("Failed to spawn child process");

    let result = tailwindcss_process.wait().expect("TAILWIND FAILED");

    if !result.success() {
        panic!("TAILWIND EXITED WITH: {:?}", result);
    }
}
