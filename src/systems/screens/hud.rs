use crate::content;
use crate::ecs::{BoomerWorld, HudHandles, Phase, Screen, WeaponKind, WeaponState};
use crate::systems::common::combo_multiplier;
use crate::theme::*;
use crate::tuning;
use nalgebra_glm::lerp;
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
    let mut weapon_label = Entity::default();
    let mut ammo_rack = Entity::default();
    let mut wave_label = Entity::default();
    let mut objective_label = Entity::default();
    let mut score_label = Entity::default();
    let mut combo_label = Entity::default();
    let mut crosshair = Entity::default();
    let mut status_label = Entity::default();
    let mut hint_label = Entity::default();
    let mut damage_overlay = Entity::default();
    let mut low_health_overlay = Entity::default();

    tree.in_parent(root, |tree| {
        low_health_overlay = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(TRANSPARENT)
            .entity();

        damage_overlay = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(TRANSPARENT)
            .entity();

        health_label = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(30.0, -54.0)),
                Ab(vec2(360.0, 40.0)),
                Anchor::BottomLeft,
            )
            .with_text("100", 40.0)
            .text_left()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
            .color_raw::<UiBase>(HEALTH)
            .entity();

        weapon_label = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-30.0, -84.0)),
                Ab(vec2(360.0, 26.0)),
                Anchor::BottomRight,
            )
            .with_text("SHOTGUN", 20.0)
            .text_right()
            .color_raw::<UiBase>(ACCENT)
            .entity();

        ammo_label = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-30.0, -54.0)),
                Ab(vec2(360.0, 40.0)),
                Anchor::BottomRight,
            )
            .with_text("40", 40.0)
            .text_right()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
            .color_raw::<UiBase>(AMMO)
            .entity();

        ammo_rack = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-30.0, -22.0)),
                Ab(vec2(420.0, 22.0)),
                Anchor::BottomRight,
            )
            .with_text("", 18.0)
            .text_right()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 1.5)
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();

        score_label = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 20.0)),
                Ab(vec2(520.0, 40.0)),
                Anchor::TopCenter,
            )
            .with_text("0", 34.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 1.5)
            .color_raw::<UiBase>(WHITE)
            .entity();

        combo_label = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 60.0)),
                Ab(vec2(360.0, 28.0)),
                Anchor::TopCenter,
            )
            .with_text("", 22.0)
            .text_center()
            .with_visible(false)
            .color_raw::<UiBase>(ACCENT_HOT)
            .entity();

        wave_label = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(30.0, 24.0)),
                Ab(vec2(280.0, 26.0)),
                Anchor::TopLeft,
            )
            .with_text("WAVE 1", 22.0)
            .text_left()
            .color_raw::<UiBase>(ACCENT)
            .entity();

        objective_label = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(30.0, 52.0)),
                Ab(vec2(420.0, 24.0)),
                Anchor::TopLeft,
            )
            .with_text("", 18.0)
            .text_left()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 1.5)
            .with_visible(false)
            .color_raw::<UiBase>(ACCENT_HOT)
            .entity();

        crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(7.0, 7.0)), Anchor::Center)
            .with_rect(3.5, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(CROSSHAIR)
            .entity();

        status_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 40.0)), Ab(vec2(780.0, 80.0)), Anchor::Center)
            .with_text("", 60.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.5)
            .with_visible(false)
            .color_raw::<UiBase>(WHITE)
            .entity();

        hint_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 53.0)), Ab(vec2(720.0, 32.0)), Anchor::Center)
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
        weapon_label,
        ammo_rack,
        wave_label,
        objective_label,
        score_label,
        combo_label,
        status_label,
        hint_label,
        crosshair,
        damage_overlay,
        low_health_overlay,
    }
}

pub fn update(boomer_world: &BoomerWorld, world: &mut World) {
    let hud = boomer_world.resources.ui_handles.hud;
    let screen = boomer_world.resources.screen.current;
    let phase = boomer_world.resources.game.phase;
    let game = &boomer_world.resources.game;
    let in_game = matches!(screen, Screen::InGame);
    let playing = in_game && matches!(phase, Phase::Playing);
    let elapsed = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    ui_set_text(
        world,
        hud.health_label,
        &format!("{:.0}", boomer_world.resources.stats.health.max(0.0)),
    );
    let weapon = &boomer_world.resources.weapon;
    ui_set_text(
        world,
        hud.ammo_label,
        &format!("{}", weapon.ammo(weapon.current)),
    );
    ui_set_text(world, hud.weapon_label, weapon.current.name());
    ui_set_text(world, hud.ammo_rack, &weapon_rack(weapon));
    ui_set_text(world, hud.score_label, &format!("{}", game.score));
    let level = &boomer_world.resources.level;
    let definition = content::level(level.index);
    let level_name = if level.custom {
        "CUSTOM"
    } else if level.story {
        crate::campaign::mission(boomer_world.resources.story.mission).title
    } else {
        definition.name
    };
    ui_set_text(
        world,
        hud.wave_label,
        &format!(
            "LEVEL {}: {}   WAVE {}/{}",
            level.index + 1,
            level_name,
            level.wave,
            level.wave_count
        ),
    );

    let show_objective = playing && level.story;
    ui_set_visible(world, hud.objective_label, show_objective);
    if show_objective {
        let text = if level.exit_active {
            "REACH THE GATE"
        } else {
            match level.objective {
                crate::campaign::Objective::Exterminate => "CLEAR THE HORDE",
                crate::campaign::Objective::Reach => "REACH THE GATE",
                crate::campaign::Objective::Boss => "KILL THE WARLORD",
                crate::campaign::Objective::Keycard => {
                    if game.has_key {
                        "REACH THE GATE"
                    } else {
                        "FIND THE KEYCARD"
                    }
                }
            }
        };
        ui_set_text(world, hud.objective_label, &format!("> {text}"));
    }

    let score_lit = lerp(
        &WHITE,
        &AMMO,
        (game.score_flash / tuning::SCORE_FLASH_TIME).clamp(0.0, 1.0),
    );
    set_color(world, hud.score_label, score_lit);

    let show_combo = playing && game.combo > 1;
    ui_set_visible(world, hud.combo_label, show_combo);
    if show_combo {
        ui_set_text(
            world,
            hud.combo_label,
            &format!("x{}  {} KILLS", combo_multiplier(game.combo), game.combo),
        );
    }

    ui_set_visible(world, hud.crosshair, playing);
    let hit = (boomer_world.resources.weapon.hit_marker / 0.12).clamp(0.0, 1.0);
    set_color(
        world,
        hud.crosshair,
        lerp(&CROSSHAIR, &vec4(1.0, 0.4, 0.3, 1.0), hit),
    );

    let dead = in_game && matches!(phase, Phase::Dead);
    let reach = level.story && matches!(level.objective, crate::campaign::Objective::Reach);
    let exit_open = playing && level.exit_active && level.banner > 0.0;
    let intro = playing && level.banner > 0.0 && !exit_open;
    let show_status = dead || exit_open || intro;
    ui_set_visible(world, hud.status_label, show_status);
    ui_set_visible(world, hud.hint_label, show_status);
    if dead {
        ui_set_text(world, hud.status_label, "YOU DIED");
        ui_set_text(
            world,
            hud.hint_label,
            &format!(
                "SCORE {}     BEST {}     R / (A) TO RETRY",
                game.score, game.best_score
            ),
        );
        set_color(world, hud.status_label, HEALTH);
    } else if exit_open {
        let (status_text, hint_text) = if reach {
            ("REACH THE GATE", "RUN TO THE GREEN GATE")
        } else {
            ("LEVEL CLEAR", "REACH THE GREEN GATE")
        };
        ui_set_text(world, hud.status_label, status_text);
        ui_set_text(world, hud.hint_label, hint_text);
        set_color(world, hud.status_label, vec4(0.4, 1.0, 0.6, 1.0));
    } else if intro {
        ui_set_text(world, hud.status_label, level_name);
        ui_set_text(world, hud.hint_label, &format!("LEVEL {}", level.index + 1));
        set_color(world, hud.status_label, ACCENT);
    }

    let flash = (game.damage_flash / tuning::DAMAGE_FLASH_TIME).clamp(0.0, 1.0);
    set_color(
        world,
        hud.damage_overlay,
        vec4(
            DAMAGE_FLASH.x,
            DAMAGE_FLASH.y,
            DAMAGE_FLASH.z,
            DAMAGE_FLASH.w * flash,
        ),
    );

    let health_fraction =
        boomer_world.resources.stats.health / boomer_world.resources.stats.max_health;
    let lowness = if playing && health_fraction < tuning::LOW_HEALTH_FRACTION {
        ((tuning::LOW_HEALTH_FRACTION - health_fraction) / tuning::LOW_HEALTH_FRACTION)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let pulse = 0.35 + 0.25 * (elapsed * 6.0).sin();
    set_color(
        world,
        hud.low_health_overlay,
        vec4(0.6, 0.0, 0.0, lowness * pulse),
    );
}

fn weapon_rack(weapon: &WeaponState) -> String {
    let segment = |kind: WeaponKind, tag: &str| {
        let count = weapon.ammo(kind);
        if weapon.current == kind {
            format!("[{tag} {count}]")
        } else {
            format!("{tag} {count}")
        }
    };
    format!(
        "{}   {}   {}",
        segment(WeaponKind::Shotgun, "1 SG"),
        segment(WeaponKind::Nailgun, "2 NG"),
        segment(WeaponKind::Rocket, "3 RL"),
    )
}

fn set_color(world: &mut World, entity: Entity, color: Vec4) {
    if let Some(node_color) = world.ui.get_ui_node_color_mut(entity) {
        node_color.colors[UiBase::INDEX] = Some(color);
    }
}
