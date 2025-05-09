use std::{
    error::Error,
    f64::consts::{PI, TAU},
};

use krpc_mars::RPCClient;

use crate::services::space_center::{self, Orbit, Vessel};

/// Calculate when the vessel will pass the given orbit
/// Both vessel and orbit must have the same primary body
/// Vessel must be at equator
/// Returns timestamp of intersection
pub fn intersect(
    client: &mut RPCClient,
    vessel: &Vessel,
    orbit: &Orbit,
) -> Result<f64, Box<dyn Error>> {
    let vessel_orbit = vessel.get_orbit().mk_call(client)?;
    let vessel_lan = vessel_orbit
        .get_longitude_of_ascending_node()
        .mk_call(client)?;
    let vessel_aop = vessel_orbit.get_argument_of_periapsis().mk_call(client)?;
    let vessel_lng = vessel_lan + vessel_aop + PI;
    let is_retro = orbit.get_inclination().mk_call(client)? < 0.0;
    let mut asc_lng = orbit.get_longitude_of_ascending_node().mk_call(client)?;
    if is_retro {
        asc_lng += PI;
    }
    let mut delta_lng = asc_lng - vessel_lng;
    if delta_lng <= 0.0 {
        delta_lng += TAU;
    }
    let body = orbit.get_body().mk_call(client)?;
    let delta_time = body.get_rotational_period().mk_call(client)? * (delta_lng / TAU);
    let timestamp = space_center::get_ut().mk_call(client)? + delta_time;
    Ok(timestamp)
}
