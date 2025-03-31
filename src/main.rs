use std::process::exit;

use krpc_client::services::space_center::SpaceCenter;
use krpc_client::Client;
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

    let auto_pilot = ship.get_auto_pilot().await?;
    auto_pilot.target_pitch_and_heading(90.0, 90.0).await?;
    auto_pilot.engage().await?;

    control.activate_next_stage().await?;

    let flight = ship.flight(None).await?;

    let alt = flight.get_surface_altitude_stream().await?;

    loop {
        alt.wait().await;
        if alt.get().await? > 100.0 {
            break;
        }
    }

    let pitch_ctrl = Interpolate::new((100.0, 32000.0), (90.0, 0.0));
    let aoa = flight.get_angle_of_attack_stream().await?;
    let pitch = flight.get_pitch_stream().await?;

    let stage_5_resources = ship.resources_in_decouple_stage(4, false).await?;
    let solid_fuel = stage_5_resources
        .amount_stream("SolidFuel".to_string())
        .await?;

    let mut srbs_separated = false;

    let stage_4_resources = ship.resources_in_decouple_stage(3, false).await?;
    let oxidizer = stage_4_resources
        .amount_stream("Oxidizer".to_string())
        .await?;

    let mut booster_separated = false;

    loop {
        aoa.wait().await;
        let alt = alt.get().await?;
        if !srbs_separated && solid_fuel.get().await? < 0.1 {
            control.activate_next_stage().await?;
            srbs_separated = true;
        }
        if !booster_separated && oxidizer.get().await? < 0.1 {
            control.activate_next_stage().await?;
            booster_separated = true;
        }
        let pitch = if alt < 24000.0 {
            let true_pitch = pitch.get().await?;
            let tgt_pitch = pitch_ctrl.inter(alt);
            let max_aoa = 5.0;
            let vel_pitch = true_pitch - aoa.get().await?;
            let min_pitch = vel_pitch - max_aoa;
            (tgt_pitch).max(min_pitch)
        } else {
            pitch_ctrl.inter(alt)
        };
        auto_pilot.target_pitch_and_heading(pitch, 90.0).await?;
        println!("AoA: {:.2}", aoa.get().await?);
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
