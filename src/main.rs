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
    auto_pilot.target_pitch_and_heading(90.0, 0.0).await?;
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

    loop {
        auto_pilot.target_pitch_and_heading(90.0, 0.0).await?;
    }

    Ok(())
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
        (((self.to.1 - self.to.0) / (self.from.1 - self.from.0)) * (from - self.from.0) + self.to.0)
            as f32
    }
}

#[cfg(test)]
mod test {
    use crate::Interpolate;

    #[test]
    fn test_interpolate() {
        let inter = Interpolate::new((2.0, 2.0), (4.0, 4.0));
        assert_eq!(inter.inter(3.0), 3.0);
    }
}
