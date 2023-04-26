mod openvpn;
use std::error::Error;

use crate::openvpn::OpenVPN;

fn main() -> Result<(), Box<dyn Error>> {
    let cfgs = OpenVPN::load_cfg()?;
    OpenVPN::connect(cfgs)?;
    Ok(())
}
