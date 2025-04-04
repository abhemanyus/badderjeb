use std::error::Error;

use krpc_mars::{krpc::Event, RPCClient, StreamClient};

use crate::services::{
    krpc::{add_event, Expression},
    space_center::{self, Vessel},
};

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
    let deltav = node.get_delta_v().mk_call(client)?;
    let burn_time = burn_time(client, ship, deltav)?;
    let burn_start_time = ut_node - burn_time / 2.0;
    space_center::warp_to(burn_start_time - 60.0, 100000.0, 2.0).mk_call(client)?;
    auto_pilot
        .set_target_direction(
            node.burn_vector(auto_pilot.get_reference_frame().mk_call(client)?)
                .mk_call(client)?,
        )
        .mk_call(client)?;
    auto_pilot.engage().mk_call(client)?;
    auto_pilot.wait().mk_call(client)?;
    let ut_time = space_center::get_ut().to_stream().mk_call(client)?;
    loop {
        let update = stream_client.recv_update()?;
        if let Some(val) = update.get_result(&ut_time)? {
            if val >= burn_start_time {
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
            if val > prev_dv {
                break;
            }
            prev_dv = val;
        }
    }
    control.set_throttle(0.0).mk_call(client)?;
    auto_pilot.disengage().mk_call(client)?;
    Ok(())
}

pub fn burn_time(
    client: &mut RPCClient,
    ship: &Vessel,
    deltav: f64,
) -> Result<f64, Box<dyn Error>> {
    let thrust = ship.get_available_thrust().mk_call(client)? as f64;
    let isp = ship.get_specific_impulse().mk_call(client)? as f64 * 9.82;
    let m0 = ship.get_mass().mk_call(client)? as f64;
    let m1 = m0 / (deltav / isp).exp();
    let flow_rate = thrust / isp;
    let burn_time = (m0 - m1) / flow_rate;
    Ok(burn_time)
}

pub fn alarm(client: &mut RPCClient, ut_time: f64) -> Result<Event, Box<dyn Error>> {
    let call = Expression::call(space_center::get_ut().get_call().clone()).mk_call(client)?;
    let exp = Expression::greater_than_or_equal(
        call,
        Expression::constant_float(ut_time as f32).mk_call(client)?,
    )
    .mk_call(client)?;
    let event = add_event(exp).mk_call(client)?;
    Ok(event)
}
