use brimstone_core::Brimstone;
use nightshade::prelude::launch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Brimstone::default())?;
    Ok(())
}
