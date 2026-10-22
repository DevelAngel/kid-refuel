use std::process::Command;

fn main() {
    if let Ok(head) = Command::new("git")
        .args(["rev-parse", "--git-path", "HEAD"])
        .output()
        && head.status.success()
        && let Ok(head) = String::from_utf8(head.stdout)
    {
        println!("cargo:rerun-if-changed={}", head.trim());
    }

    if let Ok(reference) = Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
        && reference.status.success()
        && let Ok(reference) = String::from_utf8(reference.stdout)
        && let Ok(reference_path) = Command::new("git")
            .args(["rev-parse", "--git-path", reference.trim()])
            .output()
        && reference_path.status.success()
        && let Ok(reference_path) = String::from_utf8(reference_path.stdout)
    {
        println!("cargo:rerun-if-changed={}", reference_path.trim());
    }

    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        && output.status.success()
        && let Ok(commit) = String::from_utf8(output.stdout)
    {
        let commit = commit.trim();
        if !commit.is_empty() {
            println!("cargo:rustc-env=KID_GIT_COMMIT={commit}");
        }
    }
}
