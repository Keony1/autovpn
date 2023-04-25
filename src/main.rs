use std::{
    io::{self, Read, Write},
    process::{Command, Stdio},
};

use std::error::Error;

fn disconnect_session(session_path: &str) -> Result<(), Box<dyn Error>> {
    let path: Vec<&str> = session_path.trim().split(' ').collect();
    let disconnect_cmd = Command::new("openvpn3")
        .args([
            "session-manage",
            "--session-path",
            path.get(1).unwrap(),
            "--disconnect",
        ])
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    let output = String::from_utf8(disconnect_cmd.stdout)?;
    println!("{output}");

    return Ok(());
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("openvpn3")
        .arg("sessions-list")
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdout = child.stdout.take().unwrap();

    let mut output = String::new();
    let _ = stdout.read_to_string(&mut output)?;

    let output: Vec<&str> = output.split('\n').collect();
    let session_path = output
        .iter()
        .map(|text| *text)
        .find(|text| text.contains("Path: "));

    if session_path.is_some() {
        disconnect_session(session_path.unwrap())?;
        return Ok(());
    }

    let mut child = Command::new("openvpn3")
        .stdin(Stdio::piped())
        .args([
            "session-start",
            "--config",
            "PATH FOR OVPN FILE",
        ])
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();

    stdin.write_all(b"login\n")?;
    stdin.write_all(b"password\n")?;

    let mut code = String::new();
    io::stdin().read_line(&mut code)?;

    stdin.write_all(code.as_bytes())?;
    stdin.flush()?;

    let _ = child.wait()?.success();

    Ok(())
}
