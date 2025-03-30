use std::{sync::Arc, time::Duration};

use krpc_client::{
    error::RpcError,
    services::space_center::{SpaceCenter, Vessel},
};
use tokio::{sync::RwLock, time::sleep};

impl Ship {
    pub async fn new(vessel: Vessel, center: SpaceCenter) -> Self {
        Arc::new(ShipInner {
            vessel,
            center,
            stats: Default::default(),
            controls: Default::default(),
        })
    }
    pub async fn init(&self) {
        tokio::task::spawn(async {
            self.read_alt().await;
        });
    }
    async fn read_alt(&self) -> Result<(), RpcError> {
        let alt_stream = self
            .vessel
            .flight(None)
            .await?
            .get_surface_altitude_stream()
            .await?;

        loop {
            alt_stream.wait().await;
            let alt_read = alt_stream.get().await?;
            let mut alt = self.stats.alt.write().await;
            *alt = alt_read;
        }
    }
    async fn read_pitch(&self) -> Result<(), RpcError> {
        let control = self.vessel.get_control().await?;
        loop {
            let pitch_read = *(self.controls.pitch.read().await);
            control.set_pitch(pitch_read).await?;
            sleep(Duration::from_millis(500)).await;
        }
    }
}

pub type Ship = Arc<ShipInner>;

struct ShipInner {
    vessel: Vessel,
    center: SpaceCenter,
    stats: Stats,
    controls: Controls,
}

#[derive(Default)]
struct Stats {
    alt: RwLock<f64>,
}

#[derive(Default)]
struct Controls {
    pitch: RwLock<f32>,
    head: RwLock<f32>,
}
