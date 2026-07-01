use crate::ecs::{CobaltWorld, Phase, Screen};
use crate::systems;
use nightshade::ecs::camera::systems::first_person_camera_look_system;
use nightshade::ecs::world::commands::capture_screenshot_to_path;
use nightshade::prelude::*;

/// Offline level-preview harness: `COBALT_SHOT=<idx>` (append `a` for an angled
/// bird's-eye) builds that level, frames a camera over it, saves a PNG, exits.
struct Shot {
    index: usize,
    angled: bool,
    frame: u32,
}

#[derive(Default)]
pub struct Brimstone {
    pub cobalt_world: CobaltWorld,
    shot: Option<Shot>,
}

impl State for Brimstone {
    fn initialize(&mut self, world: &mut World) {
        world.resources.window.title = "BRIMSTONE".to_string();
        if let Ok(spec) = std::env::var("COBALT_SHOT") {
            self.shot = Some(self.shot_init(world, &spec));
            return;
        }
        systems::lifecycle::initialize(&mut self.cobalt_world, world);
    }

    fn run_systems(&mut self, world: &mut World) {
        if let Some(shot) = self.shot.as_mut() {
            shot.frame += 1;
            if shot.frame == 40 {
                let suffix = if shot.angled { "a" } else { "" };
                capture_screenshot_to_path(world, format!("level_{}{}.png", shot.index, suffix));
            }
            if shot.frame >= 50 {
                world.resources.window.should_exit = true;
            }
            return;
        }
        self.run_game(world);
    }
}

impl Brimstone {
    fn shot_init(&mut self, world: &mut World, spec: &str) -> Shot {
        let spec = spec.trim();
        let angled = spec.ends_with('a') || spec.ends_with('A');
        let index: usize = spec.trim_end_matches(['a', 'A']).parse().unwrap_or(0);

        world.resources.user_interface.enabled = false;
        world.resources.retained_ui.enabled = false;
        world.resources.physics.enabled = false;

        systems::world::textures::load(world);
        systems::world::player::spawn(&mut self.cobalt_world, world);
        let definition = crate::content::level(index);
        systems::world::level::build(&mut self.cobalt_world, world, definition);
        world.resources.render_settings.fog = None;

        if let Some(player) = self.cobalt_world.resources.player.player_entity
            && let Some(transform) = world.core.get_local_transform_mut(player)
        {
            transform.translation = nalgebra_glm::vec3(0.0, 0.0, 0.0);
        }
        if let Some(camera) = self.cobalt_world.resources.player.camera_entity {
            let reach = definition.half_x.max(definition.half_z);
            let (position, rotation) = if angled {
                (
                    nalgebra_glm::vec3(0.0, reach * 1.05, definition.half_z + 5.0),
                    nalgebra_glm::quat_angle_axis(-0.78, &nalgebra_glm::vec3(1.0, 0.0, 0.0)),
                )
            } else {
                (
                    nalgebra_glm::vec3(0.0, reach * 1.5, 0.0),
                    nalgebra_glm::quat_angle_axis(
                        -std::f32::consts::FRAC_PI_2,
                        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
                    ),
                )
            };
            if let Some(transform) = world.core.get_local_transform_mut(camera) {
                transform.translation = position;
                transform.rotation = rotation;
            }
            mark_local_transform_dirty(world, camera);
        }

        Shot {
            index,
            angled,
            frame: 0,
        }
    }

    fn run_game(&mut self, world: &mut World) {
        systems::input::handle_global(&mut self.cobalt_world, world);
        systems::screens::title::handle_input(&mut self.cobalt_world, world);
        systems::screens::level_select::handle_input(&mut self.cobalt_world, world);
        systems::screens::mission_select::handle_input(&mut self.cobalt_world, world);
        systems::screens::pause::handle_input(&mut self.cobalt_world, world);
        systems::screens::cutscene::handle_input(&mut self.cobalt_world, world);

        if matches!(self.cobalt_world.resources.screen.current, Screen::Editor) {
            systems::editor::update(&mut self.cobalt_world, world);
            systems::world::fx::tick(&mut self.cobalt_world, world);
        }

        if matches!(self.cobalt_world.resources.screen.current, Screen::InGame) {
            let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
            let playing = matches!(self.cobalt_world.resources.game.phase, Phase::Playing);
            let frozen = {
                let game = &mut self.cobalt_world.resources.game;
                if game.hitstop > 0.0 {
                    game.hitstop -= delta;
                    true
                } else {
                    false
                }
            };

            let sim_active = playing && !frozen;
            self.cobalt_world.resources.player.sim_active = sim_active;
            if sim_active {
                systems::world::player::pre_look(&self.cobalt_world, world);
                first_person_camera_look_system(world);
                systems::world::player::movement(&mut self.cobalt_world, world);
                systems::world::weapon::update(&mut self.cobalt_world, world);
                systems::world::enemies::update(&mut self.cobalt_world, world);
                systems::world::projectiles::update(&mut self.cobalt_world, world);
                systems::world::pickups::update(&mut self.cobalt_world, world);
                systems::world::game::tick(&mut self.cobalt_world, world);
            }

            systems::world::player::apply_camera_feel(&mut self.cobalt_world, world);
            systems::world::billboard::update(&mut self.cobalt_world, world);
            systems::world::fx::tick(&mut self.cobalt_world, world);
            systems::world::viewmodel::update(&mut self.cobalt_world, world);
        }

        crate::adventure::update(&mut self.cobalt_world, world);

        systems::world::audio::tick(&mut self.cobalt_world, world);
        systems::screens::hud::update(&self.cobalt_world, world);
        systems::screens::cutscene::update(&mut self.cobalt_world, world);
    }
}
