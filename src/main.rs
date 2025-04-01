use services::space_center;

mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;

    let vessel = client.mk_call(&space_center::get_active_vessel())?;
    println!("Active vessel: {:?}", vessel);

    let crew = client.mk_call(&vessel.get_crew())?;
    let name = client.mk_call(&crew.first().unwrap().get_name())?;
    println!("Crew: {}", name);

    println!("Connected!");

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
