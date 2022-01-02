use std::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

pub enum Cmd {
    MoveRobot(i8, i8),
    StopRobot,
    Led(bool, bool, bool),
    ServoAbsolute(f32),
    TrackSensor,
    Buzzer(f32),
}

pub type ParseResult = Result<Cmd, ParseError>;

impl FromStr for Cmd {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult {
        let mut iter = s.split_whitespace().peekable();

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
                    if args.is_err() {
                        return Err(ParseError::InvalidArg(args.unwrap_err().to_string()));
                    }
                    let args = args.unwrap();
                    Ok(Cmd::Led(args[0], args[1], args[2]))
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
                    if pw.is_err() {
                        return Err(ParseError::InvalidArg(args[0].to_string()));
                    }
                    Ok(Cmd::ServoAbsolute(pw.unwrap()))
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
                    if pw.is_err() {
                        return Err(ParseError::InvalidArg(args[0].to_string()));
                    }
                    Ok(Cmd::Buzzer(pw.unwrap()))
                }

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
