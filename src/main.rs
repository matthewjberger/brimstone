use boom_core::Boomer;
use nightshade::prelude::launch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Boomer::default())?;
    Ok(())
}
