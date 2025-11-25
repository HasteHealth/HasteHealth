#[cfg(windows)]
const NPM: &str = "npm.cmd";
#[cfg(not(windows))]
const NPM: &str = "npm";

#[cfg(windows)]
const NPX: &str = "npx.cmd";
#[cfg(not(windows))]
const NPX: &str = "npx";

fn main() {
    println!("cargo::rerun-if-changed=css");
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=public");

    let mut npm_install = std::process::Command::new(NPM)
        .arg("i")
        .spawn()
        .expect("Failed to install node packages.");

    let npm_result = npm_install
        .wait()
        .expect("Failed to install node packages.");

    if !npm_result.success() {
        panic!("NPM INSTALL EXITED WITH: {:?}", npm_result);
    }

    let mut tailwindcss_process = std::process::Command::new(NPX)
        .arg("@tailwindcss/cli")
        .args(["-i", "./css/app.css"])
        .args(["-o", "./public/css/app.css"])
        .args(["--minify"])
        .spawn()
        .expect("Failed to spawn child process");

    let result = tailwindcss_process.wait().expect("TAILWIND FAILED");

    if !result.success() {
        panic!("TAILWIND EXITED WITH: {:?}", result);
    }
}
