use std::{
    error::Error,
    f64::consts::{PI, TAU},
};

use krpc_mars::RPCClient;

use crate::services::space_center::{self, CelestialBody, Vessel};

/// Calculate timestamp for hohmann burn to intercept target
/// Run after circularization and plane-correction
pub fn intercept(
    client: &mut RPCClient,
    vessel: &Vessel,
    target: &CelestialBody,
) -> Result<f64, Box<dyn Error>> {
    let vessel_orbit = vessel.get_orbit().mk_call(client)?;
    let vessel_lan = vessel_orbit
        .get_longitude_of_ascending_node()
        .mk_call(client)?;
    let vessel_aop = vessel_orbit.get_argument_of_periapsis().mk_call(client)?;
    let a1 = vessel_lan + vessel_aop;
    let target_orbit = target.get_orbit().mk_call(client)?;
    let target_lan = target_orbit
        .get_longitude_of_ascending_node()
        .mk_call(client)?;
    let target_aop = target_orbit.get_argument_of_periapsis().mk_call(client)?;
    let a2 = target_lan + target_aop;

    let p1 = vessel_orbit.get_period().mk_call(client)?;
    let p2 = target_orbit.get_period().mk_call(client)?;

    let aa1 = TAU / p1;
    let aa2 = TAU / p2;

    let apsis = target_orbit.get_semi_major_axis().mk_call(client)?;
    let peri = vessel_orbit.get_semi_major_axis().mk_call(client)?;
    let primary = vessel_orbit.get_body().mk_call(client)?;
    let mu = primary.get_gravitational_parameter().mk_call(client)?;

    let semi = (apsis + peri) / 2.0;
    let t = PI * (semi.powi(3) / mu).sqrt();

    let mut eta = -1.0;
    let c = PI - t * aa2;
    let mut orbit = 0;

    while eta < 0.0 {
        let extra = p1 * (orbit as f64);
        let a2 = a2 + aa2 * p1;
        eta = extra + (c + a1 - a2) / (aa2 - aa1);
        orbit += 1;
    }

    let timestamp = space_center::get_ut().mk_call(client)? + eta;
    node(client, vessel, apsis, peri, timestamp)?;
    Ok(timestamp)
}

fn node(
    client: &mut RPCClient,
    ship: &Vessel,
    apsis: f64,
    peri: f64,
    node_ut: f64,
) -> Result<(), Box<dyn Error>> {
    let vessel_orbit = ship.get_orbit().mk_call(client)?;
    let delta_v = burn(client, ship, apsis, peri)?;
    let control = ship.get_control().mk_call(client)?;
    control
        .add_node(node_ut, delta_v as f32, 0.0, 0.0)
        .mk_call(client)?;
    Ok(())
}

fn burn(
    client: &mut RPCClient,
    ship: &Vessel,
    apsis: f64,
    peri: f64,
) -> Result<f64, Box<dyn Error>> {
    let orbit = ship.get_orbit().mk_call(client)?;
    let mu = orbit
        .get_body()
        .mk_call(client)?
        .get_gravitational_parameter()
        .mk_call(client)?;
    let r = peri;
    let a1 = peri;
    let a2 = (apsis + peri) / 2.0;
    let v1 = (mu * ((2.0 / r) - (1.0 / a1))).sqrt();
    let v2 = (mu * ((2.0 / r) - (1.0 / a2))).sqrt();
    let delta_v = v2 - v1;
    Ok(delta_v)
}
