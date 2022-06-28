use roland::gpio::cleanup;

fn main() -> Result<(), anyhow::Error> {
    cleanup()
}
