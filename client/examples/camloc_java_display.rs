use std::net::SocketAddr;

use anyhow::Result;
use roblib::{
    camloc::{PlacedCamera, Position},
    event::{CamlocConnect, CamlocDisconnect, CamlocInfoUpdate, CamlocPosition},
};
use roblib_client::{transports::tcp::Tcp, Robot};
use std::io::{stderr, Write};

fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Robot::new(Tcp::connect(ip)?);

    robot.subscribe(CamlocPosition, on_position)?;
    robot.subscribe(CamlocConnect, on_connect)?;
    robot.subscribe(CamlocDisconnect, on_disconnect)?;
    robot.subscribe(CamlocInfoUpdate, on_info_update)?;

    loop {
        std::thread::park();
    }
}

fn on_position(position: Position) -> Result<()> {
    println!("{position}");

    let mut se = stderr();
    se.write_all(
        &[
            0i32.to_be_bytes().as_slice(),
            position.x.to_be_bytes().as_slice(),
            position.y.to_be_bytes().as_slice(),
            position.rotation.to_be_bytes().as_slice(),
        ]
        .concat(),
    )?;

    Ok(())
}

fn on_connect((address, camera): (SocketAddr, PlacedCamera)) -> Result<()> {
    let address = address.to_string();
    println!("New camera connected from {address}");

    let mut se = stderr();
    se.write_all(
        &[
            1i32.to_be_bytes().as_slice(),
            (address.len() as u16).to_be_bytes().as_slice(),
            address.as_bytes(),
            camera.position.x.to_be_bytes().as_slice(),
            camera.position.y.to_be_bytes().as_slice(),
            camera.position.rotation.to_be_bytes().as_slice(),
            camera.fov.to_be_bytes().as_slice(),
        ]
        .concat(),
    )?;
    Ok(())
}

fn on_disconnect(address: SocketAddr) -> Result<()> {
    let address = address.to_string();
    println!("Camera disconnected from {address}");

    let mut se = stderr();

    se.write_all(
        &[
            2i32.to_be_bytes().as_slice(),
            (address.len() as u16).to_be_bytes().as_slice(),
            address.as_bytes(),
        ]
        .concat(),
    )?;

    Ok(())
}

fn on_info_update((address, camera): (SocketAddr, PlacedCamera)) -> Result<()> {
    let address = address.to_string();

    let mut se = stderr();
    se.write_all(
        &[
            3i32.to_be_bytes().as_slice(),
            (address.len() as u16).to_be_bytes().as_slice(),
            address.as_bytes(),
            camera.position.x.to_be_bytes().as_slice(),
            camera.position.y.to_be_bytes().as_slice(),
            camera.position.rotation.to_be_bytes().as_slice(),
            camera.fov.to_be_bytes().as_slice(),
        ]
        .concat(),
    )?;

    Ok(())
}
