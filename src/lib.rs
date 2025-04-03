#![feature(macro_metavar_expr)]

pub mod interpolate;
pub mod services;
pub mod vector;

#[macro_export]
macro_rules! key_to_call {
    (alt, $flight: expr, $orbit: expr) => {
        &$flight.get_surface_altitude()
    };
    (aoa, $flight: expr, $orbit: expr) => {
        &$flight.get_angle_of_attack()
    };
    (pitch, $flight: expr, $orbit: expr) => {
        &$flight.get_pitch()
    };
    (apop, $flight: expr, $orbit: expr) => {
        &$orbit.get_apoapsis_altitude()
    };
    (perip, $flight: expr, $orbit: expr) => {
        &$orbit.get_periapsis_altitude()
    };
    (eta_apop, $flight: expr, $orbit: expr) => {
        &$orbit.get_time_to_apoapsis()
    };
}
#[macro_export]
macro_rules! streamer {
    ($client: expr, $vessel: expr, {$($key: ident $val: ty),*}) => {
        {
        use krpc_mars::{batch_call_unwrap, stream::StreamHandle, RPCClient, StreamClient};
        use crate::services::space_center::Vessel;
        use std::error::Error;
        struct Streamer {
            $(
                $key: StreamHandle<$val>,
            )*
        }
        impl Streamer {
            fn init(client: &mut RPCClient, vessel: &Vessel) -> Result<Self, Box<dyn Error>>{
                let flight = vessel
                    .flight(vessel.get_reference_frame().mk_call(client)?)
                    .mk_call(client)?;
                let orbit = vessel.get_orbit().mk_call(client)?;
                let calls = batch_call_unwrap!(
                    client,
                    (
                        $(
                            &key_to_call!($key, flight, orbit).to_stream(),
                        )*
                    )
                )?;
                Ok(Self {
                        $(
                            $key: calls.${index()},
                        )*
                })
            }
        }
        pub struct Attitude {
            _streamer: Streamer,
            $(
                pub $key: $val,
            )*
        }
        impl Attitude {
            pub fn init(client: &mut RPCClient, vessel: &Vessel) -> Result<Self, Box<dyn Error>> {
                Ok(Self {
                    _streamer: Streamer::init(client, vessel)?,
                    $(
                        $key: Default::default(),
                    )*
                })
            }
            pub fn update(&mut self, stream_client: &mut StreamClient) -> Result<(), Box<dyn Error>> {
                let update = stream_client.recv_update()?;
                $(
                    if let Some(val) = update.get_result(&self._streamer.$key)? {
                        self.$key = val;
                    }
                )*
                Ok(())
            }
        }
        Attitude::init($client, $vessel)
    }
    };
}

#[cfg(test)]
mod test {
    use krpc_mars::RPCClient;

    use crate::services::space_center;

    #[test]
    fn test_streamer() {
        let mut client = RPCClient::connect("Test", "127.0.0.1:5000").unwrap();
        let vessel = space_center::get_active_vessel()
            .mk_call(&mut client)
            .unwrap();
        let attitude = streamer!(&mut client, &vessel, {alt f64, pitch f32});
    }
}
