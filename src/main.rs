mod interpolate;
mod services;

use interpolate::*;
use krpc_mars::{stream::StreamHandle, RPCClient, StreamClient};
use services::space_center::{self, Vessel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;
    let mut stream_client = krpc_mars::StreamClient::connect(&client, "127.0.0.1:50001")?;

    let vessel = client.mk_call(&space_center::get_active_vessel())?;
    println!("Active vessel: {:?}", vessel);

    let crew = client.mk_call(&vessel.get_crew())?;
    let name = client.mk_call(&crew.first().unwrap().get_name())?;
    println!("Crew: {}", name);

    let mut attitude = Attitude::default();

    let streamer = Streamer::init(&mut client, &vessel);

    let (control, auto_pilot) = krpc_mars::batch_call_unwrap!(
        &mut client,
        (&vessel.get_control(), &vessel.get_auto_pilot(),)
    )?;

    control.set_sas(false).mk_call(&mut client)?;
    auto_pilot
        .target_pitch_and_heading(90.0, 90.0)
        .mk_call(&mut client)?;
    auto_pilot.engage().mk_call(&mut client)?;

    control.set_throttle(1.0).mk_call(&mut client)?;
    control.activate_next_stage().mk_call(&mut client)?;

    let pitch_ctrl = Interpolate::new((1000.0, 32000.0), (90.0, 0.0));

    loop {
        streamer.update(&mut stream_client, &mut attitude);
        auto_pilot
            .target_pitch_and_heading(pitch_ctrl.inter(attitude.alt), 90.0)
            .mk_call(&mut client)?;
    }
}

#[derive(Default)]
struct Attitude {
    pitch: f32,
    aoa: f32,
    alt: f64,
    apop: f64,
    peri: f64,
    eta_apop: f64,
}

struct Streamer {
    pitch: StreamHandle<f32>,
    aoa: StreamHandle<f32>,
    alt: StreamHandle<f64>,
    apop: StreamHandle<f64>,
    peri: StreamHandle<f64>,
    eta_apop: StreamHandle<f64>,
}

impl Streamer {
    fn init(client: &mut RPCClient, vessel: &Vessel) -> Self {
        let flight = vessel
            .flight(vessel.get_reference_frame().mk_call(client).unwrap())
            .mk_call(client)
            .unwrap();
        let orbit = vessel.get_orbit().mk_call(client).unwrap();
        let streams = krpc_mars::batch_call_unwrap!(
            client,
            (
                &flight.get_pitch().to_stream(),
                &flight.get_angle_of_attack().to_stream(),
                &flight.get_surface_altitude().to_stream(),
                &orbit.get_apoapsis_altitude().to_stream(),
                &orbit.get_periapsis().to_stream(),
                &orbit.get_time_to_apoapsis().to_stream(),
            )
        )
        .unwrap();
        Self {
            pitch: streams.0,
            aoa: streams.1,
            alt: streams.2,
            apop: streams.3,
            peri: streams.4,
            eta_apop: streams.5,
        }
    }

    fn update(&self, stream_client: &mut StreamClient, attitude: &mut Attitude) {
        let update = stream_client.recv_update().unwrap();
        let results = (
            update.get_result(&self.pitch).unwrap(),
            update.get_result(&self.aoa).unwrap(),
            update.get_result(&self.alt).unwrap(),
            update.get_result(&self.apop).unwrap(),
            update.get_result(&self.peri).unwrap(),
            update.get_result(&self.eta_apop).unwrap(),
        );

        if let Some(val) = results.0 {
            attitude.pitch = val
        };
        if let Some(val) = results.1 {
            attitude.aoa = val
        };
        if let Some(val) = results.2 {
            attitude.alt = val
        };
        if let Some(val) = results.3 {
            attitude.apop = val
        };
        if let Some(val) = results.4 {
            attitude.peri = val
        };
        if let Some(val) = results.5 {
            attitude.eta_apop = val
        };
    }
}
