use std::error::Error;

use krpc_mars::{RPCClient, StreamClient};

use crate::services::space_center::{self, Vessel};

pub fn maneuver(
    client: &mut RPCClient,
    stream_client: &mut StreamClient,
    ship: &Vessel,
) -> Result<(), Box<dyn Error>> {
    let control = ship.get_control().mk_call(client)?;
    control.set_throttle(0.0).mk_call(client)?;
    let auto_pilot = ship.get_auto_pilot().mk_call(client)?;
    let node = control
        .get_nodes()
        .mk_call(client)?
        .into_iter()
        .next()
        .ok_or("No node found!")?;
    let ut_node = node.get_ut().mk_call(client)?;
    space_center::warp_to(ut_node - 60.0, 10.0, 2.0).mk_call(client)?;
    let rf = ship.get_orbital_reference_frame().mk_call(client)?;
    auto_pilot.set_reference_frame(rf);
    auto_pilot
        .set_target_direction(node.burn_vector(rf).mk_call(client)?)
        .mk_call(client)?;
    auto_pilot.engage().mk_call(client)?;
    auto_pilot.wait().mk_call(client)?;
    let ut_time = space_center::get_ut().to_stream().mk_call(client)?;
    loop {
        let update = stream_client.recv_update()?;
        if let Some(val) = update.get_result(&ut_time)? {
            if val >= ut_node {
                break;
            }
        }
    }
    control.set_throttle(1.0).mk_call(client)?;
    let remaining_dv = node.get_remaining_delta_v().to_stream().mk_call(client)?;
    let mut prev_dv = f64::MAX;
    loop {
        let update = stream_client.recv_update()?;
        if let Some(val) = update.get_result(&remaining_dv)? {
            if val >= prev_dv {
                break;
            }
            prev_dv = val;
        }
    }
    control.set_throttle(0.0).mk_call(client)?;
    auto_pilot.disengage().mk_call(client)?;
    Ok(())
}
