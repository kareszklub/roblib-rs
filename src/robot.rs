// TODO
// just some placeholder code to test the server.
// actual robot code is coming soon.

pub fn move_robot(left: i8, right: i8) {
    info!("Moving robot: {}:{}", left, right);
}

pub fn stop_robot() {
    info!("Stopping robot");
}

pub fn led(r: bool, g: bool, b: bool) {
    info!("LED: {}:{}:{}", r, g, b);
}

pub fn servo_absolute(degree: f32) {
    info!("Servo absolute: {}", degree);
}

pub type SensorData = [i32; 4];
pub fn track_sensor() -> SensorData {
    info!("Track sensor");
    [0, 0, 0, 0]
}

pub fn buzzer(pw: f32) {
    info!("Buzzer: {}", pw);
}
