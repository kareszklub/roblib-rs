use env_logger::fmt::Color;
use log::Level;
use std::{env, io::Write};

/// custom `log` format
pub fn init_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "actix_web=info,roblib_rs=info");
    }

    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_bold(true);
            let message = style.value(record.args());

            let timestamp = buf.timestamp();

            let mut style = buf.style();
            let level = match record.level() {
                Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
                Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
                Level::Info => style.set_color(Color::Green).value("INFO"),
                Level::Warn => style.set_color(Color::Yellow).value("WARN"),
                Level::Error => style.set_color(Color::Red).value("ERROR"),
            };

            writeln!(buf, "[{}]({}): {}", level, timestamp, message)
        })
        .init();
}

/// custom `actix-web` logger format
pub fn actix_log() -> actix_web::middleware::Logger {
    actix_web::middleware::Logger::new("%{METHOD}xi %U %s %Dms %{r}a %{User-Agent}i")
        .custom_request_replace("METHOD", |req| req.method().to_string())
}
