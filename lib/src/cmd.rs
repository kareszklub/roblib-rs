use std::{
    fmt::{self, Debug, Display},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

pub enum Cmd {
    /// m
    MoveRobot(i8, i8),
    /// s
    StopRobot,
    /// l
    Led(bool, bool, bool),
    /// v
    ServoAbsolute(f32),
    /// t
    TrackSensor,
    /// b
    Buzzer(f32),
    /// z
    GetTime,
}

pub type ParseResult = Result<Cmd, ParseError>;

impl FromStr for Cmd {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult {
        let mut iter = s.split_whitespace();

        if let Some(cmd) = iter.next() {
            return match cmd {
                "m" => {
                    let args = iter.collect::<Vec<_>>();
                    let len = args.len();
                    if len < 2 {
                        return Err(ParseError::MissingArg);
                    }
                    if len > 2 {
                        return Err(ParseError::InvalidSyntax);
                    }
                    let dirs = (args[0].parse::<i8>(), args[1].parse::<i8>());
                    if dirs.0.is_err() {
                        return Err(ParseError::InvalidArg(args[0].to_string()));
                    }
                    if dirs.1.is_err() {
                        return Err(ParseError::InvalidArg(args[1].to_string()));
                    }
                    Ok(Cmd::MoveRobot(dirs.0.unwrap(), dirs.1.unwrap()))
                }
                "s" => Ok(Cmd::StopRobot),
                "l" => {
                    let args = iter.collect::<Vec<_>>();
                    let len = args.len();
                    if len < 3 {
                        return Err(ParseError::MissingArg);
                    }
                    if len > 3 {
                        return Err(ParseError::InvalidSyntax);
                    }
                    let args = args
                        .iter()
                        .map(|s| match *s {
                            "1" => Ok(true),
                            "0" => Ok(false),
                            _ => Err(ParseError::InvalidArg(s.to_string())),
                        })
                        .collect::<Result<Vec<_>, _>>();
                    match args {
                        Ok(args) => Ok(Cmd::Led(args[0], args[1], args[2])),
                        Err(e) => Err(e),
                    }
                }
                "v" => {
                    let args = iter.collect::<Vec<_>>();
                    let len = args.len();
                    if len < 1 {
                        return Err(ParseError::MissingArg);
                    }
                    if len > 1 {
                        return Err(ParseError::InvalidSyntax);
                    }
                    let pw = args[0].parse::<f32>();
                    match pw {
                        Ok(pw) => Ok(Cmd::ServoAbsolute(pw)),
                        Err(_) => Err(ParseError::InvalidArg(args[0].to_string())),
                    }
                }
                "t" => Ok(Cmd::TrackSensor),
                "b" => {
                    let args = iter.collect::<Vec<_>>();
                    let len = args.len();
                    if len < 1 {
                        return Err(ParseError::MissingArg);
                    }
                    if len > 1 {
                        return Err(ParseError::InvalidSyntax);
                    }
                    let pw = args[0].parse::<f32>();
                    match pw {
                        Ok(pw) => Ok(Cmd::Buzzer(pw)),
                        Err(_) => Err(ParseError::InvalidArg(args[0].to_string())),
                    }
                }
                "z" => Ok(Cmd::GetTime),

                _ => Err(ParseError::InvalidCommand(cmd.to_string())),
            };
        }
        Err(ParseError::InvalidSyntax)
    }
}

pub enum ParseError {
    InvalidSyntax,
    InvalidCommand(String),
    MissingArg,
    InvalidArg(String),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidSyntax => f.write_str("InvalidSyntax"),
            ParseError::InvalidCommand(cmd) => write!(f, "InvalidCommand: {}", cmd),
            ParseError::MissingArg => f.write_str("MissingArg"),
            ParseError::InvalidArg(arg) => write!(f, "InvalidArg: {}", arg),
        }
    }
}
impl Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidSyntax => f.write_str("InvalidSyntax"),
            ParseError::InvalidCommand(cmd) => write!(f, "InvalidCommand: {}", cmd),
            ParseError::MissingArg => f.write_str("MissingArg"),
            ParseError::InvalidArg(arg) => write!(f, "InvalidArg: {}", arg),
        }
    }
}
impl std::error::Error for ParseError {}

pub type SensorData = [i32; 4];

pub fn get_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_micros() as f64
        / 1000f64
}
