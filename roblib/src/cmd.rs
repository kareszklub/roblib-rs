use roblib_macro::Parsable;

#[derive(Parsable)]
pub enum Cmd {
    #[cfg(feature = "roland")]
    MoveRobot(f64, f64),

    #[cfg(feature = "roland")]
    #[prefix = 'M']
    MoveRobotByAngle(f64, f64),

    #[cfg(feature = "roland")]
    StopRobot,

    #[cfg(feature = "roland")]
    Led(bool, bool, bool),

    #[cfg(feature = "roland")]
    #[prefix = 'a']
    ServoAbsolute(f64),

    #[cfg(feature = "roland")]
    Buzzer(f64),

    #[cfg(feature = "roland")]
    #[query = "[bool; 4]"]
    TrackSensor,

    #[cfg(feature = "roland")]
    #[query = "f64"]
    UltraSensor,

    #[cfg(feature = "gpio")]
    #[query = "bool"]
    ReadPin(u8),

    #[cfg(feature = "gpio")]
    #[prefix = 'p']
    SetPin(u8, bool),

    #[cfg(feature = "gpio")]
    #[prefix = 'w']
    SetPwm(u8, f64, f64),

    #[cfg(feature = "gpio")]
    #[prefix = 'V']
    ServoBasic(u8, f64),

    #[cfg(feature = "camloc")]
    #[prefix = 'P']
    #[query = "Option<camloc_server::Position>"]
    GetPosition,

    Nop,

    #[prefix = 'U']
    GetUptime,
}

pub const ARGUMENT_SEPARATOR: char = ' ';

pub trait Parsable: Sized {
    fn read_from_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self>;
    fn write_str(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn read_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self>;
    fn write_binary(&self, w: &mut impl std::io::Write) -> anyhow::Result<()>;
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Parsable::write_str(self, f)
    }
}

impl std::str::FromStr for Cmd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parsable::read_from_str(&mut s.split(ARGUMENT_SEPARATOR))
    }
}

#[cfg(feature = "camloc")]
impl Parsable for camloc_server::Position {
    fn read_from_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        use anyhow::Error;
        Ok(Self {
            x: s.next()
                .ok_or_else(|| Error::msg("No x"))?
                .parse()
                .map_err(|_| Error::msg("Couldn't parse x"))?,
            y: s.next()
                .ok_or_else(|| Error::msg("No y"))?
                .parse()
                .map_err(|_| Error::msg("Couldn't parse y"))?,
            rotation: s
                .next()
                .ok_or_else(|| Error::msg("No rotation"))?
                .parse()
                .map_err(|_| Error::msg("Couldn't parse rotation"))?,
        })
    }

    fn write_str(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.x)?;
        write!(f, "{}", self.y)?;
        write!(f, "{}", self.rotation)
    }

    fn read_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut f = [0; 8];
        r.read_exact(&mut f)?;
        let x = f64::from_be_bytes(f);
        let mut f = [0; 8];
        r.read_exact(&mut f)?;
        let y = f64::from_be_bytes(f);
        let mut f = [0; 8];
        r.read_exact(&mut f)?;
        let rotation = f64::from_be_bytes(f);
        Ok(Self { x, y, rotation })
    }

    fn write_binary(&self, w: &mut impl std::io::Write) -> anyhow::Result<()> {
        w.write_all(&self.x.to_be_bytes())?;
        w.write_all(&self.y.to_be_bytes())?;
        w.write_all(&self.rotation.to_be_bytes())?;

        Ok(())
    }
}
