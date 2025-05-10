use betterjeb::{intercept::intercept, maneuver::maneuver, services::space_center};
use krpc_mars::{RPCClient, StreamClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RPCClient::connect("kRPC TEST", "127.0.0.1:50000")?;
    let mut stream_client = StreamClient::connect(&client, "127.0.0.1:50001")?;

    let ship = space_center::get_active_vessel().mk_call(&mut client)?;

    let target_orbit = space_center::get_target_body()
        .mk_call(&mut client)?
        .get_orbit()
        .mk_call(&mut client);
    // .or(space_center::get_target_body()
    //     .mk_call(&mut client)?
    //     .get_orbit()
    //     .mk_call(&mut client));
    if let Ok(_) = target_orbit {
        let target = space_center::get_target_body().mk_call(&mut client)?;
        intercept(&mut client, &ship, &target)?;
    }
    maneuver(&mut client, &mut stream_client, &ship)?;
    Ok(())
}
