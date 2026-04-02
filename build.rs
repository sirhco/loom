fn main() {
    // Embed git commit hash at build time
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=CONCOCT_GIT_HASH={hash}");
        }
    }

    // Re-run if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
