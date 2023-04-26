use std::{
    fs::{self, File},
    io::{self, stdout, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

const OPENVPN3_CONFIG_FILE: &str = "openvpn3_cfgs.txt";

pub struct OpenVPN;
pub struct OpenVPNConfigs {
    user: String,
    pass: String,
    ovpn_file_path: String,
}

impl OpenVPN {
    pub fn load_cfg() -> io::Result<OpenVPNConfigs> {
        if Path::new(OPENVPN3_CONFIG_FILE).exists() {
            let contents = fs::read_to_string(OPENVPN3_CONFIG_FILE)?;
            let mut configs = contents.split("\n");
            let user = configs.next().unwrap();
            let pass = configs.next().unwrap();
            let path = configs.next().unwrap();

            return Ok(OpenVPNConfigs {
                user: user.into(),
                pass: pass.into(),
                ovpn_file_path: path.into(),
            });
        }

        let (user, pass, path) = OpenVPN::create_cfg_file()?;
        Ok(OpenVPNConfigs {
            user,
            pass,
            ovpn_file_path: path,
        })
    }

    fn create_cfg_file() -> io::Result<(String, String, String)> {
        let mut user = String::new();
        let mut pass = String::new();
        let mut path = String::new();

        print!("Digite o usuário OpenVPN: ");
        let stdin = io::stdin();
        stdout().flush()?;
        stdin.read_line(&mut user)?;
        print!("Digite a senha OpenVPN: ");
        stdout().flush()?;
        stdin.read_line(&mut pass)?;
        print!("Digite o path para o arquivo de configuração OpenVPN: ");
        stdout().flush()?;
        let _ = stdin.read_line(&mut path);

        let mut config_file = File::create(OPENVPN3_CONFIG_FILE)?;
        config_file.write(user.as_bytes())?;
        config_file.write(pass.as_bytes())?;
        config_file.write(path.as_bytes())?;

        let user = user.trim();
        let pass = pass.trim();
        let path = path.trim();
        Ok((user.into(), pass.into(), path.into()))
    }

    pub fn connect(auth: OpenVPNConfigs) -> io::Result<()> {
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
            OpenVPN::disconnect(session_path.unwrap())?;
            return Ok(());
        }

        let mut child = Command::new("openvpn3")
            .stdin(Stdio::piped())
            .args(["session-start", "--config", &auth.ovpn_file_path])
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();

        stdin.write_all(format!("{}\n", auth.user).as_bytes())?;
        stdin.write_all(format!("{}\n", auth.pass).as_bytes())?;

        let mut code = String::new();
        io::stdin().read_line(&mut code)?;

        stdin.write_all(code.as_bytes())?;
        stdin.flush()?;

        let _ = child.wait()?.success();
        Ok(())
    }

    fn disconnect(session_path: &str) -> io::Result<()> {
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
        let output = String::from_utf8(disconnect_cmd.stdout).unwrap();
        println!("{output}");

        Ok(())
    }
}
