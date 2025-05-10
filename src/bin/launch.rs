use betterjeb::{
    circ::circ, intersect::intersect, launch::launch, maneuver::maneuver, services::space_center,
};
use krpc_mars::{RPCClient, StreamClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RPCClient::connect("kRPC TEST", "127.0.0.1:50000")?;
    let mut stream_client = StreamClient::connect(&client, "127.0.0.1:50001")?;

    let ship = space_center::get_active_vessel().mk_call(&mut client)?;

    let mut inclination = 0.0;

    let target_orbit = space_center::get_target_vessel()
        .mk_call(&mut client)?
        .get_orbit()
        .mk_call(&mut client)
        .or(space_center::get_target_body()
            .mk_call(&mut client)?
            .get_orbit()
            .mk_call(&mut client));
    if let Ok(target_orbit) = target_orbit {
        inclination = target_orbit
            .get_inclination()
            .mk_call(&mut client)?
            .to_degrees();
        let intersect_time = intersect(&mut client, &ship, &target_orbit)?;
        space_center::warp_to(intersect_time, 100000.0, 2.0).mk_call(&mut client)?;
    }

    launch(&mut client, &mut stream_client, &ship, inclination as f32)?;
    circ(&mut client, &ship)?;
    maneuver(&mut client, &mut stream_client, &ship)?;
    Ok(())
}
