use anyhow::Result;
use assert_cmd::Command;

#[test]
fn short_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("unfold")?;
    for text in [
        "Usage:",
        "Arguments:",
        "Options:",
        "--follow-to-source",
        "--num-layers",
        "--help",
        "--version",
        "-f,",
        "-n,",
        "-h,",
        "-V,",
    ] {
        cmd.arg("-h")
            .assert()
            .stdout(predicates::str::contains(text));
    }
    Ok(())
}
