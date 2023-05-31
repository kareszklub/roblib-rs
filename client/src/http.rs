use anyhow::{anyhow, Result};
use awc::Client;
use roblib::cmd::{get_time, parse_track_sensor_data, Cmd, SensorData};

pub struct Robot {
    base_url: String,
    client: Client,
}
impl Robot {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: format!("{base_url}/cmd"),
            client: Client::default(),
        }
    }

    /// Send a raw command.
    /// You probably don't need this.
    pub async fn send<'a>(&self, cmd: String) -> Result<String> {
        let mut req = match self.client.post(&self.base_url).send_body(cmd).await {
            Ok(x) => x,
            Err(_) => return Err(anyhow!("failed to connect")),
        };

        Ok(String::from_utf8(req.body().await?.to_vec())?)
    }

    pub async fn cmd(&self, cmd: Cmd) -> Result<String> {
        let s = cmd.to_string();
        debug!("S: {}", &s);
        let r = self.send(s).await?;
        debug!("R: {}", &r);
        Ok(r)
    }

    #[cfg(feature = "roland")]
    pub async fn get_sensor_data(&self) -> Result<SensorData> {
        parse_track_sensor_data(&self.cmd(Cmd::TrackSensor).await?)
    }

    pub async fn measure_latency(&self) -> Result<f64> {
        let start = get_time()?;
        self.cmd(Cmd::GetTime).await?;
        Ok(get_time()? - start)
    }
}
