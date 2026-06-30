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
        systems::screens::level_select::handle_input(&mut self.boomer_world, world);
        systems::screens::mission_select::handle_input(&mut self.boomer_world, world);
        systems::screens::pause::handle_input(&mut self.boomer_world, world);
        systems::screens::cutscene::handle_input(&mut self.boomer_world, world);

        if matches!(self.boomer_world.resources.screen.current, Screen::Editor) {
            systems::editor::update(&mut self.boomer_world, world);
            systems::world::fx::tick(&mut self.boomer_world, world);
        }

        if matches!(self.boomer_world.resources.screen.current, Screen::InGame) {
            let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
            let playing = matches!(self.boomer_world.resources.game.phase, Phase::Playing);
            let frozen = {
                let game = &mut self.boomer_world.resources.game;
                if game.hitstop > 0.0 {
                    game.hitstop -= delta;
                    true
                } else {
                    false
                }
            };

            if playing && !frozen {
                systems::world::player::pre_look(&self.boomer_world, world);
                first_person_camera_look_system(world);
                systems::world::player::movement(&mut self.boomer_world, world);
                systems::world::weapon::update(&mut self.boomer_world, world);
                systems::world::enemies::update(&mut self.boomer_world, world);
                systems::world::projectiles::update(&mut self.boomer_world, world);
                systems::world::pickups::update(&mut self.boomer_world, world);
                systems::world::game::tick(&mut self.boomer_world, world);
            }

            systems::world::player::apply_camera_feel(&mut self.boomer_world, world);
            systems::world::billboard::update(&mut self.boomer_world, world);
            systems::world::fx::tick(&mut self.boomer_world, world);
        }

        systems::world::audio::tick(&mut self.boomer_world, world);
        systems::screens::hud::update(&self.boomer_world, world);
        systems::screens::cutscene::update(&self.boomer_world, world);
    }
}
