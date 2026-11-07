use std::ffi::OsStr;
use std::io::Write;
use std::process::{Command, Output, Stdio};

pub fn output<I, S>(args: I) -> std::io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("sudo")
        .args(args)
        .stderr(Stdio::piped())
        .output()
}

pub fn run<I, S>(args: I) -> Result<(), Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = output(args)?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "privileged command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

pub fn write_file(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("sudo")
        .args(["tee", path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("privileged writer has no stdin")?
        .write_all(content.as_bytes())?;
    if child.wait()?.success() {
        return Ok(());
    }
    Err(format!("privileged write failed path={}", path).into())
}
