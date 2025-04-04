use std::error::Error;

use krpc_mars::{batch_call_unwrap, RPCClient, StreamClient};

use crate::{
    maneuver::maneuver,
    services::space_center::{self, Vessel},
};

pub fn circ(
    client: &mut RPCClient,
    stream_client: &mut StreamClient,
    ship: &Vessel,
) -> Result<(), Box<dyn Error>> {
    let orbit = ship.get_orbit().mk_call(client)?;
    let apop = orbit.get_apoapsis().mk_call(client)?;
    let peri = orbit.get_periapsis().mk_call(client)?;
    println!("Apoapsis: {apop}");
    println!("Periapsis: {peri}");

    let node_ut;
    let apsis;
    if apop < 0.0 {
        // circ at periapsis
        apsis = peri;
        node_ut = orbit.get_time_to_periapsis();
    } else {
        // circ at apoapsis
        apsis = apop;
        node_ut = orbit.get_time_to_apoapsis();
    }
    let delta_v = circ_burn(client, ship, apsis)?;
    let (ut, node_ut) = batch_call_unwrap!(client, (&space_center::get_ut(), &node_ut))?;
    let node_time = ut + node_ut;
    let control = ship.get_control().mk_call(client)?;
    control
        .add_node(node_time, delta_v as f32, 0.0, 0.0)
        .mk_call(client)?;
    maneuver(client, stream_client, ship)?;
    Ok(())
}

fn circ_burn(client: &mut RPCClient, ship: &Vessel, apsis: f64) -> Result<f64, Box<dyn Error>> {
    let orbit = ship.get_orbit().mk_call(client)?;
    let mu = orbit
        .get_body()
        .mk_call(client)?
        .get_gravitational_parameter()
        .mk_call(client)?;
    let r = apsis;
    let a1 = orbit.get_semi_major_axis().mk_call(client)?;
    let a2 = r;
    let v1 = (mu * ((2.0 / r) - (1.0 / a1))).sqrt();
    let v2 = (mu * ((2.0 / r) - (1.0 / a2))).sqrt();
    let delta_v = v2 - v1;
    Ok(delta_v)
}
