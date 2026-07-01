use crate::ecs::{BrimstoneWorld, Phase, Screen};
use crate::systems;
use nightshade::ecs::camera::systems::first_person_camera_look_system;
use nightshade::prelude::*;

#[derive(Default)]
pub struct Brimstone {
    pub brimstone_world: BrimstoneWorld,
    #[cfg(not(target_arch = "wasm32"))]
    shot: Option<DevShot>,
}

impl State for Brimstone {
    fn initialize(&mut self, world: &mut World) {
        world.resources.window.title = "BRIMSTONE".to_string();
        systems::lifecycle::initialize(&mut self.brimstone_world, world);
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var("BRIMSTONE_SHOT").is_ok() {
            crate::adventure::open(&mut self.brimstone_world, world);
            self.shot = Some(DevShot::new());
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        self.run_game(world);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(mut shot) = self.shot.take() {
            shot.player_camera = self.brimstone_world.resources.player.camera_entity;
            let done = shot.run(world);
            if !done {
                self.shot = Some(shot);
            }
        }
    }
}

impl Brimstone {
    fn run_game(&mut self, world: &mut World) {
        systems::input::handle_global(&mut self.brimstone_world, world);
        systems::screens::title::handle_input(&mut self.brimstone_world, world);
        systems::screens::level_select::handle_input(&mut self.brimstone_world, world);
        systems::screens::mission_select::handle_input(&mut self.brimstone_world, world);
        systems::screens::pause::handle_input(&mut self.brimstone_world, world);
        systems::screens::cutscene::handle_input(&mut self.brimstone_world, world);

        if matches!(
            self.brimstone_world.resources.screen.current,
            Screen::Editor
        ) {
            systems::editor::update(&mut self.brimstone_world, world);
            systems::world::fx::tick(&mut self.brimstone_world, world);
        }

        if matches!(
            self.brimstone_world.resources.screen.current,
            Screen::InGame
        ) {
            let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
            let playing = matches!(self.brimstone_world.resources.game.phase, Phase::Playing);
            let frozen = {
                let game = &mut self.brimstone_world.resources.game;
                if game.hitstop > 0.0 {
                    game.hitstop -= delta;
                    true
                } else {
                    false
                }
            };

            let sim_active = playing && !frozen;
            self.brimstone_world.resources.player.sim_active = sim_active;
            if sim_active {
                systems::world::player::pre_look(&self.brimstone_world, world);
                first_person_camera_look_system(world);
                systems::world::player::movement(&mut self.brimstone_world, world);
                systems::world::weapon::update(&mut self.brimstone_world, world);
                systems::world::enemies::update(&mut self.brimstone_world, world);
                systems::world::projectiles::update(&mut self.brimstone_world, world);
                systems::world::pickups::update(&mut self.brimstone_world, world);
                systems::world::game::tick(&mut self.brimstone_world, world);
            }

            systems::world::player::apply_camera_feel(&mut self.brimstone_world, world);
            systems::world::billboard::update(&mut self.brimstone_world, world);
            systems::world::fx::tick(&mut self.brimstone_world, world);
            update_vfx_system(world);
            systems::world::viewmodel::update(&mut self.brimstone_world, world);
        }

        crate::adventure::update(&mut self.brimstone_world, world);

        systems::world::audio::tick(&mut self.brimstone_world, world);
        systems::screens::hud::update(&self.brimstone_world, world);
        systems::screens::cutscene::update(&mut self.brimstone_world, world);
    }
}

/// Offline overworld-preview harness. `BRIMSTONE_SHOT=1` boots straight into the
/// adventure overworld, lets the terrain stream in, then flies a free camera to a
/// fixed tour of vantage points and saves a PNG at each before exiting. Set
/// `BRIMSTONE_SHOT_DIR` to choose the output directory (defaults to the working
/// directory). Desktop-only so the wasm build never references the screenshot API.
#[cfg(not(target_arch = "wasm32"))]
struct DevShot {
    camera: Option<Entity>,
    player_camera: Option<Entity>,
    poses: Vec<(Vec3, f32, f32)>,
    output_dir: String,
    warmup: u32,
    frame: u32,
}

#[cfg(not(target_arch = "wasm32"))]
impl DevShot {
    const WARMUP_FRAMES: u32 = 230;
    const FRAMES_PER_POSE: u32 = 14;
    const CAPTURE_AT: u32 = 6;

    fn new() -> Self {
        Self {
            camera: None,
            player_camera: None,
            poses: vec![
                (Vec3::new(0.0, 6.0, 44.0), -0.08, 0.0),
                (Vec3::new(0.0, 78.0, 165.0), -0.42, 0.0),
                (Vec3::new(0.0, 230.0, 30.0), -1.45, 0.0),
                (Vec3::new(180.0, 70.0, 180.0), -0.35, -0.785),
                (Vec3::new(200.0, 48.0, 210.0), -0.2, 0.0),
            ],
            output_dir: std::env::var("BRIMSTONE_SHOT_DIR").unwrap_or_else(|_| ".".to_string()),
            warmup: Self::WARMUP_FRAMES,
            frame: 0,
        }
    }

    fn ensure_camera(&mut self, world: &mut World) -> Entity {
        if let Some(camera) = self.camera {
            return camera;
        }
        let camera = spawn_camera(
            world,
            Vec3::new(0.0, 60.0, 120.0),
            "DevShot Camera".to_string(),
        );
        if let Some(camera_data) = world.core.get_camera_mut(camera) {
            camera_data.projection = Projection::Perspective(PerspectiveCamera {
                y_fov_rad: 62.0_f32.to_radians(),
                z_near: 0.1,
                z_far: Some(4000.0),
                aspect_ratio: None,
            });
        }
        world.resources.active_camera = Some(camera);
        self.camera = Some(camera);
        camera
    }

    /// Returns true when the tour is finished and the app should exit.
    fn run(&mut self, world: &mut World) -> bool {
        self.frame += 1;
        if self.frame <= self.warmup {
            return false;
        }
        let elapsed = self.frame - self.warmup - 1;
        // The first slot renders the player's own first-person camera, to confirm
        // the character is standing on the streamed terrain; the rest fly the free
        // tour camera to the fixed vantage points.
        let slot = (elapsed / Self::FRAMES_PER_POSE) as usize;
        if slot > self.poses.len() {
            world.resources.window.should_exit = true;
            return true;
        }
        let local = elapsed % Self::FRAMES_PER_POSE;
        if slot == 0 {
            if let Some(player_camera) = self.player_camera {
                world.resources.active_camera = Some(player_camera);
            }
            if local == Self::CAPTURE_AT {
                let path = format!("{}/overworld_player.png", self.output_dir);
                nightshade::ecs::world::commands::capture_screenshot_to_path(world, path);
            }
            return false;
        }
        let pose_index = slot - 1;
        let camera = self.ensure_camera(world);
        let (position, pitch, yaw) = self.poses[pose_index];
        if let Some(transform) = world.core.get_local_transform_mut(camera) {
            transform.translation = position;
            transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::new(0.0, 1.0, 0.0))
                * nalgebra_glm::quat_angle_axis(pitch, &Vec3::new(1.0, 0.0, 0.0));
        }
        world.resources.active_camera = Some(camera);
        mark_local_transform_dirty(world, camera);
        if local == Self::CAPTURE_AT {
            let path = format!("{}/overworld_{}.png", self.output_dir, pose_index);
            nightshade::ecs::world::commands::capture_screenshot_to_path(world, path);
        }
        false
    }
}
