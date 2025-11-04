fn main() {
    println!("cargo::rerun-if-changed=css");
    println!("cargo::rerun-if-changed=src");

    let mut tailwindcss_process = match std::env::consts::OS {
        "macos" => std::process::Command::new("./tools/tailwindcss-macos")
            .args(["-i", "./css/app.css"])
            .args(["-o", "./public/css/app.css"])
            .spawn()
            .expect("Failed to spawn child process"),
        "windows" => panic!("Unsupported OS"),
        "linux" => {
             std::process::Command::new("./tools/tailwindcss-linux-x64")
            .args(["-i", "./css/app.css"])
            .args(["-o", "./public/css/app.css"])
            .spawn()
            .expect("Failed to spawn child process"),
        }
        _ => panic!("Unsupported OS"),
    };

    let result = tailwindcss_process.wait().expect("TAILWIND FAILED");

    if !result.success() {
        panic!("TAILWIND EXITED WITH: {:?}", result);
    }
}
