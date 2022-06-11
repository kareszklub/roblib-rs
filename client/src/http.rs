use anyhow::{anyhow, Result};
use awc::Client;
use roblib_shared::cmd::{get_time, SensorData};

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

    pub async fn move_robot(&self, left: i8, right: i8) -> Result<String> {
        let s = format!("m {} {}", left, right);
        debug!("S: {}", s);
        self.send(s).await
    }
    pub async fn stop_robot(&self) -> Result<String> {
        debug!("S: s");
        let res = self.send("s".into()).await?;
        debug!("R: {res}");
        Ok(res)
    }
    pub async fn led(&self, (r, g, b): (bool, bool, bool)) -> Result<String> {
        let s = format!("l {} {} {}", r as i8, g as i8, b as i8);
        debug!("S: {}", s);
        self.send(s).await
    }
    pub async fn servo_absolute(&self, angle: f32) -> Result<String> {
        let s = format!("v {}", angle);
        debug!("S: {}", s);
        self.send(s).await
    }
    pub async fn buzzer(&self, freq: u16) -> Result<String> {
        let s = format!("b {}", freq);
        debug!("S: {}", s);
        self.send(s).await
    }
    pub async fn get_sensor_data(&self) -> Result<SensorData> {
        debug!("S: t");
        let d = self
            .send("t".into())
            .await?
            .split(',')
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|v: Vec<_>| {
                panic!("Expected a Vec of length {} but it was {}", 4, v.len())
            });
        debug!("R {:?}", d);
        Ok(d)
    }
    pub async fn measure_latency(&self) -> Result<f64> {
        let start = get_time();
        debug!("S: ");
        let r = self.send("z".into()).await?;
        debug!("R {}", &r);
        Ok(get_time() - start)
    }
}
