use betterjeb::{circ::circ, maneuver::maneuver, services::space_center};
use krpc_mars::{RPCClient, StreamClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RPCClient::connect("kRPC TEST", "127.0.0.1:50000")?;
    let mut stream_client = StreamClient::connect(&client, "127.0.0.1:50001")?;

    let ship = space_center::get_active_vessel().mk_call(&mut client)?;

    circ(&mut client, &ship)?;
    maneuver(&mut client, &mut stream_client, &ship)?;
    Ok(())
}
