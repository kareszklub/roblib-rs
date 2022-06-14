pub fn clamp(x: i8, min: i8, max: i8) -> i8 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
