use std::error::Error;
use std::process::exit;

use krpc_client::services::space_center::{SpaceCenter, Vessel};
use krpc_client::stream::Stream;
use krpc_client::Client;
use tokio::join;
use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("kRPC TEST", "127.0.0.1", 50000, 50001).await?;

    let sc = SpaceCenter::new(client.clone());

    // Check out our vessel.
    let ship = sc.get_active_vessel().await?;

    tokio::task::spawn(async move {
        ctrl_c().await.unwrap();
        sc.revert_to_launch().await.unwrap();
        println!("Terminated!");
        exit(0);
    });

    // Initialization sequence
    let control = ship.get_control().await?;
    control.set_sas(false).await?;
    control.set_lights(true).await?;
    control.set_throttle(1.0).await?;

    let mut state = State::Launch;
    let mut attitude = Attitude::new(&ship).await?;

    let auto_pilot = ship.get_auto_pilot().await?;
    let pitch_ctrl = Interpolate::new((100.0, 32000.0), (90.0, 0.0));
    let aoa_ctrl = Interpolate::new((1000.0, 18000.0), (5.0, 25.0));

    let mut prev_state = state;

    loop {
        attitude.update().await?;
        if state != prev_state {
            println!("{prev_state:?}->{state:?}");
            prev_state = state;
        }
        state = match state {
            State::Launch => {
                auto_pilot.target_pitch_and_heading(90.0, 90.0).await?;
                auto_pilot.engage().await?;
                control.activate_next_stage().await?;
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
                auto_pilot.set_target_pitch(pitch).await?;

                if attitude.apop > 80000.0 {
                    State::Coast
                } else {
                    State::Turn
                }
            }
            State::Coast => {
                control.set_throttle(0.0).await?;
                if attitude.alt > 70000.0 {
                    State::Circ
                } else {
                    State::Coast
                }
            }
            State::Circ => {
                if attitude.pitch.abs() > 0.1 {
                    auto_pilot.set_target_pitch(0.0).await?;
                }
                if attitude.perip > 80000.0 {
                    State::Orbit
                } else if attitude.eta_apop < 10.0 {
                    control.set_throttle(1.0).await?;
                    State::Circ
                } else {
                    control.set_throttle(0.0).await?;
                    State::Circ
                }
            }
            State::Orbit => {
                control.set_throttle(0.0).await?;
                auto_pilot.disengage().await?;
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

pub struct Attitude {
    alt: f64,
    aoa: f32,
    pitch: f32,
    apop: f64,
    perip: f64,
    eta_apop: f64,
    alt_stream: Stream<f64>,
    aoa_stream: Stream<f32>,
    pitch_stream: Stream<f32>,
    apop_stream: Stream<f64>,
    eta_apop_stream: Stream<f64>,
    perip_stream: Stream<f64>,
}

impl Attitude {
    pub async fn new(ship: &Vessel) -> Result<Self, Box<dyn Error>> {
        let flight = ship.flight(None).await?;
        Ok(Self {
            alt: 0.0,
            aoa: 0.0,
            pitch: 0.0,
            apop: 0.0,
            eta_apop: 0.0,
            perip: 0.0,
            alt_stream: flight.get_surface_altitude_stream().await?,
            aoa_stream: flight.get_angle_of_attack_stream().await?,
            pitch_stream: flight.get_pitch_stream().await?,
            apop_stream: ship
                .get_orbit()
                .await?
                .get_apoapsis_altitude_stream()
                .await?,
            eta_apop_stream: ship
                .get_orbit()
                .await?
                .get_time_to_apoapsis_stream()
                .await?,
            perip_stream: ship
                .get_orbit()
                .await?
                .get_periapsis_altitude_stream()
                .await?,
        })
    }
    pub async fn update(&mut self) -> Result<(), Box<dyn Error>> {
        join!(
            self.alt_stream.wait(),
            self.aoa_stream.wait(),
            self.pitch_stream.wait(),
            self.apop_stream.wait(),
            self.eta_apop_stream.wait(),
            self.perip_stream.wait()
        );
        let (alt, aoa, pitch, apop, eta_apop, perip) = join!(
            self.alt_stream.get(),
            self.aoa_stream.get(),
            self.pitch_stream.get(),
            self.apop_stream.get(),
            self.eta_apop_stream.get(),
            self.perip_stream.get()
        );
        self.alt = alt?;
        self.aoa = aoa?;
        self.pitch = pitch?;
        self.apop = apop?;
        self.eta_apop = eta_apop?;
        self.perip = perip?;
        Ok(())
    }
}

pub struct Interpolate {
    from: (f64, f64),
    to: (f64, f64),
}

impl Interpolate {
    pub fn new(from: (f64, f64), to: (f64, f64)) -> Self {
        Self { from, to }
    }
    pub fn inter(&self, from: f64) -> f32 {
        let out = ((self.to.1 - self.to.0) / (self.from.1 - self.from.0)) * (from - self.from.0)
            + self.to.0;

        if self.to.0 < self.to.1 {
            out.max(self.to.0).min(self.to.1) as f32
        } else {
            out.max(self.to.1).min(self.to.0) as f32
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Interpolate;

    #[test]
    fn test_interpolate() {
        let inter = Interpolate::new((0.0, 4.0), (2.0, 4.0));
        assert_eq!(inter.inter(2.0), 3.0);

        let inter = Interpolate::new((-4.0, 0.0), (2.0, 4.0));
        assert_eq!(inter.inter(-2.0), 3.0);

        let inter = Interpolate::new((-4.0, 0.0), (2.0, 4.0));
        assert_eq!(inter.inter(-8.0), 2.0);

        let inter = Interpolate::new((-4.0, 0.0), (2.0, 4.0));
        assert_eq!(inter.inter(8.0), 4.0);
    }
}
