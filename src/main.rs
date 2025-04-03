use std::error::Error;

use betterjeb::{
    interpolate::Interpolate,
    services::space_center::{self, Vessel},
    streamer,
};
use krpc_mars::{batch_call_unwrap, stream::StreamHandle, RPCClient, StreamClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RPCClient::connect("kRPC TEST", "127.0.0.1:50000")?;
    let mut stream_client = StreamClient::connect(&client, "127.0.0.1:50001")?;

    let ship = space_center::get_active_vessel().mk_call(&mut client)?;

    // Initialization sequence
    let control = ship.get_control().mk_call(&mut client)?;
    control.set_sas(false).mk_call(&mut client)?;
    control.set_lights(true).mk_call(&mut client)?;
    control.set_throttle(1.0).mk_call(&mut client)?;

    let mut state = State::Launch;
    let mut attitude = Attitude::default();

    let streamer = Streamer::init(&mut client, &ship)?;

    let auto_pilot = ship.get_auto_pilot().mk_call(&mut client)?;
    let pitch_ctrl = Interpolate::new((100.0, 32000.0), (90.0, 0.0));
    let aoa_ctrl = Interpolate::new((1000.0, 18000.0), (5.0, 25.0));

    let mut prev_state = state;

    loop {
        streamer.update(&mut stream_client, &mut attitude)?;
        if state != prev_state {
            println!("{prev_state:?}->{state:?}");
            prev_state = state;
        }
        state = match state {
            State::Launch => {
                auto_pilot
                    .target_pitch_and_heading(90.0, 90.0)
                    .mk_call(&mut client)?;
                auto_pilot.engage().mk_call(&mut client)?;
                control.activate_next_stage().mk_call(&mut client)?;
                State::Ascent
            }
            State::Ascent => {
                if attitude.alt < 1000.0 {
                    State::Ascent
                } else {
                    State::Turn
                }
            }
            State::Turn => {
                let tgt_pitch = pitch_ctrl.inter(attitude.alt);
                let pitch = if attitude.alt < 24000.0 {
                    tgt_pitch.max(attitude.pitch - attitude.aoa - aoa_ctrl.inter(attitude.alt))
                } else {
                    tgt_pitch
                };
                auto_pilot.set_target_pitch(pitch).mk_call(&mut client)?;

                if attitude.apop > 80000.0 {
                    State::Coast
                } else {
                    State::Turn
                }
            }
            State::Coast => {
                control.set_throttle(0.0).mk_call(&mut client)?;
                if attitude.alt > 70000.0 {
                    State::Circ
                } else {
                    State::Coast
                }
            }
            State::Circ => {
                if attitude.pitch.abs() > 0.1 {
                    auto_pilot.set_target_pitch(0.0).mk_call(&mut client)?;
                }
                if attitude.perip > 80000.0 {
                    State::Orbit
                } else if attitude.eta_apop < 10.0 {
                    control.set_throttle(1.0).mk_call(&mut client)?;
                    State::Circ
                } else {
                    control.set_throttle(0.0).mk_call(&mut client)?;
                    State::Circ
                }
            }
            State::Orbit => {
                control.set_throttle(0.0).mk_call(&mut client)?;
                auto_pilot.disengage().mk_call(&mut client)?;
                return Ok(());
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    Launch,
    Ascent,
    Turn,
    Coast,
    Circ,
    Orbit,
}

#[derive(Default)]
pub struct Attitude {
    alt: f64,
    aoa: f32,
    pitch: f32,
    apop: f64,
    perip: f64,
    eta_apop: f64,
}

pub struct Streamer {
    alt: StreamHandle<f64>,
    aoa: StreamHandle<f32>,
    pitch: StreamHandle<f32>,
    apop: StreamHandle<f64>,
    perip: StreamHandle<f64>,
    eta_apop: StreamHandle<f64>,
}

impl Streamer {
    pub fn init(client: &mut RPCClient, vessel: &Vessel) -> Result<Self, Box<dyn Error>> {
        let flight = vessel
            .flight(vessel.get_reference_frame().mk_call(client)?)
            .mk_call(client)?;
        let orbit = vessel.get_orbit().mk_call(client)?;
        let calls = batch_call_unwrap!(
            client,
            (
                &flight.get_surface_altitude().to_stream(),
                &flight.get_angle_of_attack().to_stream(),
                &flight.get_pitch().to_stream(),
                &orbit.get_apoapsis_altitude().to_stream(),
                &orbit.get_periapsis_altitude().to_stream(),
                &orbit.get_time_to_apoapsis().to_stream(),
            )
        )?;
        Ok(Self {
            alt: calls.0,
            aoa: calls.1,
            pitch: calls.2,
            apop: calls.3,
            perip: calls.4,
            eta_apop: calls.5,
        })
    }

    pub fn update(
        &self,
        stream_client: &mut StreamClient,
        attitude: &mut Attitude,
    ) -> Result<(), Box<dyn Error>> {
        let update = stream_client.recv_update()?;
        if let Some(val) = update.get_result(&self.alt)? {
            attitude.alt = val;
        }
        if let Some(val) = update.get_result(&self.apop)? {
            attitude.apop = val;
        }
        if let Some(val) = update.get_result(&self.perip)? {
            attitude.perip = val;
        }
        if let Some(val) = update.get_result(&self.eta_apop)? {
            attitude.eta_apop = val;
        }
        if let Some(val) = update.get_result(&self.aoa)? {
            attitude.aoa = val;
        }
        if let Some(val) = update.get_result(&self.pitch)? {
            attitude.pitch = val;
        }
        Ok(())
    }
}
