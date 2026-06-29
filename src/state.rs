use crate::ecs::{BoomerWorld, Phase, Screen};
use crate::systems;
use nightshade::ecs::camera::systems::first_person_camera_look_system;
use nightshade::prelude::*;

#[derive(Default)]
pub struct Boomer {
    pub boomer_world: BoomerWorld,
}

impl State for Boomer {
    fn initialize(&mut self, world: &mut World) {
        world.resources.window.title = "Boomer".to_string();
        systems::lifecycle::initialize(&mut self.boomer_world, world);
    }

    fn run_systems(&mut self, world: &mut World) {
        systems::input::handle_global(&mut self.boomer_world, world);
        systems::screens::title::handle_input(&mut self.boomer_world, world);
        systems::screens::pause::handle_input(&mut self.boomer_world, world);

        if matches!(self.boomer_world.resources.screen.current, Screen::InGame) {
            if matches!(self.boomer_world.resources.game.phase, Phase::Playing) {
                first_person_camera_look_system(world);
                systems::world::player::movement(&mut self.boomer_world, world);
                systems::world::weapon::update(&mut self.boomer_world, world);
                systems::world::enemies::update(&mut self.boomer_world, world);
                systems::world::pickups::update(&mut self.boomer_world, world);
            }
            systems::world::flash::update(&mut self.boomer_world, world);
            systems::world::billboard::update(&mut self.boomer_world, world);
        }

        systems::world::audio::tick(&mut self.boomer_world, world);
        systems::screens::hud::update(&mut self.boomer_world, world);
    }
}
