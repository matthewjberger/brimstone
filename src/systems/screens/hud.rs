use crate::ecs::{BoomerWorld, HudHandles, Phase, Screen};
use crate::systems::world::enemies;
use crate::theme::*;
use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();

    let mut health_label = Entity::default();
    let mut ammo_label = Entity::default();
    let mut enemies_label = Entity::default();
    let mut wave_label = Entity::default();
    let mut crosshair = Entity::default();
    let mut status_label = Entity::default();
    let mut hint_label = Entity::default();
    let mut damage_overlay = Entity::default();

    tree.in_parent(root, |tree| {
        damage_overlay = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(TRANSPARENT)
            .entity();

        health_label = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(28.0, -54.0)),
                Ab(vec2(360.0, 36.0)),
                Anchor::BottomLeft,
            )
            .with_text("HEALTH 100", 28.0)
            .text_left()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.85), 1.5)
            .color_raw::<UiBase>(HEALTH)
            .entity();

        ammo_label = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-28.0, -54.0)),
                Ab(vec2(360.0, 36.0)),
                Anchor::BottomRight,
            )
            .with_text("SHELLS 24", 28.0)
            .text_right()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.85), 1.5)
            .color_raw::<UiBase>(AMMO)
            .entity();

        enemies_label = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(28.0, 22.0)),
                Ab(vec2(280.0, 28.0)),
                Anchor::TopLeft,
            )
            .with_text("IMPS 0", 22.0)
            .text_left()
            .color_raw::<UiBase>(TEXT_COLOR)
            .entity();

        wave_label = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(28.0, 52.0)),
                Ab(vec2(280.0, 24.0)),
                Anchor::TopLeft,
            )
            .with_text("WAVE 1", 18.0)
            .text_left()
            .color_raw::<UiBase>(ACCENT)
            .entity();

        crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(6.0, 6.0)), Anchor::Center)
            .with_rect(3.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(CROSSHAIR)
            .entity();

        status_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 42.0)), Ab(vec2(760.0, 72.0)), Anchor::Center)
            .with_text("", 56.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.85), 2.0)
            .with_visible(false)
            .color_raw::<UiBase>(WHITE)
            .entity();

        hint_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 54.0)), Ab(vec2(640.0, 32.0)), Anchor::Center)
            .with_text("", 24.0)
            .text_center()
            .with_visible(false)
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();
    });

    HudHandles {
        root,
        health_label,
        ammo_label,
        enemies_label,
        wave_label,
        status_label,
        hint_label,
        crosshair,
        damage_overlay,
    }
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    boomer_world.resources.game.damage_flash =
        (boomer_world.resources.game.damage_flash - delta).max(0.0);

    let hud = boomer_world.resources.ui_handles.hud;
    let screen = boomer_world.resources.screen.current;
    let phase = boomer_world.resources.game.phase;
    let in_game = matches!(screen, Screen::InGame);

    ui_set_text(
        world,
        hud.health_label,
        &format!("HEALTH {:.0}", boomer_world.resources.stats.health),
    );
    ui_set_text(
        world,
        hud.ammo_label,
        &format!("SHELLS {}", boomer_world.resources.weapon.ammo),
    );
    ui_set_text(
        world,
        hud.enemies_label,
        &format!("IMPS {}", enemies::alive_count(boomer_world)),
    );
    ui_set_text(
        world,
        hud.wave_label,
        &format!("WAVE {}", boomer_world.resources.game.wave.max(1)),
    );

    ui_set_visible(
        world,
        hud.crosshair,
        in_game && matches!(phase, Phase::Playing),
    );

    let show_status = in_game && !matches!(phase, Phase::Playing);
    ui_set_visible(world, hud.status_label, show_status);
    ui_set_visible(world, hud.hint_label, show_status);
    if show_status {
        match phase {
            Phase::Dead => {
                ui_set_text(world, hud.status_label, "YOU DIED");
                ui_set_text(world, hud.hint_label, "Press R or (A) to try again");
                set_color(world, hud.status_label, HEALTH);
            }
            Phase::Won => {
                ui_set_text(world, hud.status_label, "ARENA CLEARED");
                ui_set_text(world, hud.hint_label, "Press R or (A) to play again");
                set_color(world, hud.status_label, ACCENT);
            }
            Phase::Playing => {}
        }
    }

    let flash = (boomer_world.resources.game.damage_flash / 0.5).clamp(0.0, 1.0);
    let overlay = vec4(
        DAMAGE_FLASH.x,
        DAMAGE_FLASH.y,
        DAMAGE_FLASH.z,
        DAMAGE_FLASH.w * flash,
    );
    set_color(world, hud.damage_overlay, overlay);
}

fn set_color(world: &mut World, entity: Entity, color: Vec4) {
    if let Some(node_color) = world.ui.get_ui_node_color_mut(entity) {
        node_color.colors[UiBase::INDEX] = Some(color);
    }
}
