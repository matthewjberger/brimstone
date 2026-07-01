use crate::content;
use crate::ecs::{BrimstoneWorld, HudHandles, Phase, Screen, WeaponKind, WeaponState};
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
    let mut boss_panel = Entity::default();
    let mut boss_bar = Entity::default();
    let mut crosshair = Entity::default();
    let mut status_panel = Entity::default();
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

        // Health chip, bottom-left: dim caption over a bright value on a panel.
        let health_chip = panel(
            tree,
            Rl(vec2(0.0, 100.0)) + Ab(vec2(24.0, -92.0)),
            Ab(vec2(228.0, 68.0)),
            Anchor::BottomLeft,
            PANEL_BG_DEEP,
            true,
        );
        tree.in_parent(health_chip, |tree| {
            caption(tree, "HEALTH", Ab(vec2(16.0, 9.0)));
            health_label = tree
                .add_node()
                .window(Ab(vec2(16.0, 26.0)), Ab(vec2(196.0, 34.0)), Anchor::TopLeft)
                .with_text("100", 32.0)
                .text_left()
                .color_raw::<UiBase>(HEALTH)
                .entity();
        });

        // Ammo chip, bottom-right: weapon name, big count, and the weapon rack.
        let ammo_chip = panel(
            tree,
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-24.0, -92.0)),
            Ab(vec2(360.0, 68.0)),
            Anchor::BottomRight,
            PANEL_BG_DEEP,
            true,
        );
        tree.in_parent(ammo_chip, |tree| {
            weapon_label = tree
                .add_node()
                .window(Ab(vec2(16.0, 9.0)), Ab(vec2(200.0, 16.0)), Anchor::TopLeft)
                .with_text("SHOTGUN", 14.0)
                .text_left()
                .color_raw::<UiBase>(ACCENT)
                .entity();
            ammo_label = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 5.0)),
                    Ab(vec2(150.0, 34.0)),
                    Anchor::TopRight,
                )
                .with_text("40", 30.0)
                .text_right()
                .color_raw::<UiBase>(AMMO)
                .entity();
            ammo_rack = tree
                .add_node()
                .window(Ab(vec2(16.0, 46.0)), Ab(vec2(328.0, 16.0)), Anchor::TopLeft)
                .with_text("", 12.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity();
        });

        // Score chip, top-center.
        let score_chip = panel(
            tree,
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 16.0)),
            Ab(vec2(220.0, 54.0)),
            Anchor::TopCenter,
            PANEL_BG_DEEP,
            true,
        );
        tree.in_parent(score_chip, |tree| {
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 8.0)),
                    Ab(vec2(200.0, 12.0)),
                    Anchor::TopCenter,
                )
                .with_text("SCORE", 11.0)
                .text_center()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity();
            score_label = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 20.0)),
                    Ab(vec2(210.0, 30.0)),
                    Anchor::TopCenter,
                )
                .with_text("0", 28.0)
                .text_center()
                .color_raw::<UiBase>(WHITE)
                .entity();
        });

        // Combo flourish floats just under the score chip.
        combo_label = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 78.0)),
                Ab(vec2(360.0, 28.0)),
                Anchor::TopCenter,
            )
            .with_text("", 22.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
            .with_visible(false)
            .color_raw::<UiBase>(ACCENT_HOT)
            .entity();

        // Wave + objective chip, top-left.
        let info_chip = panel(
            tree,
            Ab(vec2(24.0, 20.0)),
            Ab(vec2(344.0, 62.0)),
            Anchor::TopLeft,
            PANEL_BG_DEEP,
            true,
        );
        tree.in_parent(info_chip, |tree| {
            wave_label = tree
                .add_node()
                .window(Ab(vec2(14.0, 9.0)), Ab(vec2(316.0, 22.0)), Anchor::TopLeft)
                .with_text("WAVE 1", 18.0)
                .text_left()
                .color_raw::<UiBase>(ACCENT)
                .entity();
            objective_label = tree
                .add_node()
                .window(Ab(vec2(14.0, 36.0)), Ab(vec2(316.0, 18.0)), Anchor::TopLeft)
                .with_text("", 15.0)
                .text_left()
                .with_visible(false)
                .color_raw::<UiBase>(ACCENT_HOT)
                .entity();
        });

        // Boss bar, top-center, panel and bar hidden until a warlord is alive.
        boss_panel = panel(
            tree,
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 84.0)),
            Ab(vec2(560.0, 34.0)),
            Anchor::TopCenter,
            PANEL_BG_DEEP,
            false,
        );
        tree.in_parent(boss_panel, |tree| {
            boss_bar = tree
                .add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(536.0, 24.0)), Anchor::Center)
                .with_text("", 20.0)
                .text_center()
                .color_raw::<UiBase>(HEALTH)
                .entity();
        });

        crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(7.0, 7.0)), Anchor::Center)
            .with_rect(3.5, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(CROSSHAIR)
            .entity();

        // Center banner (level intro / clear / death), on a deep panel so the
        // big text reads cleanly over a busy scene. Hidden until needed.
        status_panel = panel(
            tree,
            Rl(vec2(50.0, 42.0)),
            Ab(vec2(760.0, 150.0)),
            Anchor::Center,
            BACKDROP,
            false,
        );
        tree.in_parent(status_panel, |tree| {
            status_label = tree
                .add_node()
                .window(Rl(vec2(50.0, 34.0)), Ab(vec2(736.0, 72.0)), Anchor::Center)
                .with_text("", 54.0)
                .text_center()
                .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
                .color_raw::<UiBase>(WHITE)
                .entity();
            hint_label = tree
                .add_node()
                .window(Rl(vec2(50.0, 72.0)), Ab(vec2(720.0, 30.0)), Anchor::Center)
                .with_text("", 22.0)
                .text_center()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity();
        });
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
        boss_panel,
        boss_bar,
        status_panel,
        status_label,
        hint_label,
        crosshair,
        damage_overlay,
        low_health_overlay,
    }
}

/// A readable HUD chip: rounded translucent panel with an accent border and a
/// soft drop shadow, matching the engine's own paneled-HUD style.
fn panel(
    tree: &mut UiTreeBuilder,
    position: impl Into<UiValue<Vec2>>,
    size: impl Into<UiValue<Vec2>>,
    anchor: Anchor,
    fill: Vec4,
    visible: bool,
) -> Entity {
    tree.add_node()
        .window(position, size, anchor)
        .with_rect(8.0, 1.5, PANEL_BORDER)
        .color_raw::<UiBase>(fill)
        .with_shadow(vec4(0.0, 0.0, 0.0, 0.35), vec2(0.0, 2.0), 12.0, 0.0)
        .with_visible(visible)
        .entity()
}

/// A small dimmed caption parented inside a chip (top-left relative offset).
fn caption(tree: &mut UiTreeBuilder, text: &str, offset: impl Into<UiValue<Vec2>>) {
    tree.add_node()
        .window(offset, Ab(vec2(160.0, 14.0)), Anchor::TopLeft)
        .with_text(text, 12.0)
        .text_left()
        .color_raw::<UiBase>(TEXT_DIM)
        .entity();
}

pub fn update(brimstone_world: &BrimstoneWorld, world: &mut World) {
    let hud = brimstone_world.resources.ui_handles.hud;
    let screen = brimstone_world.resources.screen.current;
    let phase = brimstone_world.resources.game.phase;
    let game = &brimstone_world.resources.game;
    let in_game = matches!(screen, Screen::InGame);
    let playing = in_game && matches!(phase, Phase::Playing);
    let elapsed = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    ui_set_text(
        world,
        hud.health_label,
        &format!("{:.0}", brimstone_world.resources.stats.health.max(0.0)),
    );
    let weapon = &brimstone_world.resources.weapon;
    ui_set_text(world, hud.ammo_label, &ammo_text(weapon.current, weapon));
    ui_set_text(world, hud.weapon_label, weapon.current.name());
    ui_set_text(world, hud.ammo_rack, &weapon_rack(weapon));
    ui_set_text(world, hud.score_label, &format!("{}", game.score));
    let level = &brimstone_world.resources.level;
    let definition = content::level(level.index);
    let level_name = if level.custom {
        "CUSTOM"
    } else if level.story {
        crate::campaign::mission(brimstone_world.resources.story.mission).title
    } else {
        definition.name
    };
    ui_set_text(
        world,
        hud.wave_label,
        &format!("{}   WAVE {}/{}", level_name, level.wave, level.wave_count),
    );

    let show_objective = playing
        && (level.story || !matches!(level.objective, crate::campaign::Objective::Exterminate));
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
                        "SEIZE THE POWER CORE"
                    }
                }
            }
        };
        let compass = objective_compass(brimstone_world, world, level);
        ui_set_text(world, hud.objective_label, &format!("> {text}{compass}"));
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

    let boss_health = brimstone_world
        .resources
        .game
        .boss_entity
        .and_then(|entity| brimstone_world.get_enemy(entity))
        .map(|enemy| enemy.health);
    let show_boss = playing && boss_health.is_some();
    ui_set_visible(world, hud.boss_panel, show_boss);
    if let Some(health) = boss_health {
        let max = brimstone_world.resources.game.boss_max_health.max(1.0);
        let fraction = (health / max).clamp(0.0, 1.0);
        let filled = (fraction * 24.0).round() as usize;
        let bar: String = "|".repeat(filled) + &".".repeat(24 - filled);
        ui_set_text(world, hud.boss_bar, &format!("WARLORD  [{bar}]"));
    }

    ui_set_visible(world, hud.crosshair, playing);
    let hit = (brimstone_world.resources.weapon.hit_marker / 0.12).clamp(0.0, 1.0);
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
    ui_set_visible(world, hud.status_panel, show_status);
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
        brimstone_world.resources.stats.health / brimstone_world.resources.stats.max_health;
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

/// Ammo count for the HUD: a count for normal weapons, an infinity glyph for the
/// pistol (which never depletes).
fn ammo_text(kind: WeaponKind, weapon: &WeaponState) -> String {
    if kind.infinite() {
        "INF".to_string()
    } else {
        format!("{}", weapon.ammo(kind))
    }
}

fn weapon_rack(weapon: &WeaponState) -> String {
    let segment = |kind: WeaponKind, tag: &str| {
        let count = ammo_text(kind, weapon);
        if weapon.current == kind {
            format!("[{tag} {count}]")
        } else {
            format!("{tag} {count}")
        }
    };
    format!(
        "{}  {}  {}  {}  {}  {}",
        segment(WeaponKind::Shotgun, "1 SG"),
        segment(WeaponKind::Nailgun, "2 NG"),
        segment(WeaponKind::Rocket, "3 RK"),
        segment(WeaponKind::Railgun, "4 RG"),
        segment(WeaponKind::Pistol, "5 PS"),
        segment(WeaponKind::Tesla, "6 TS"),
    )
}

/// A directional arrow + distance to the current navigation target (the open
/// gate, or the keycard while it's still out there), relative to where the
/// player is looking. Empty when there's nothing to navigate to.
fn objective_compass(
    brimstone_world: &BrimstoneWorld,
    world: &World,
    level: &crate::ecs::LevelState,
) -> String {
    let target = if level.exit_active {
        level.exit_position
    } else if matches!(level.objective, crate::campaign::Objective::Keycard)
        && !brimstone_world.resources.game.has_key
    {
        let key = crate::campaign::mission(brimstone_world.resources.story.mission).key;
        nalgebra_glm::vec3(key[0], 0.0, key[2])
    } else {
        return String::new();
    };

    let Some(player) = brimstone_world
        .resources
        .player
        .player_entity
        .and_then(|entity| world.core.get_local_transform(entity))
        .map(|transform| transform.translation)
    else {
        return String::new();
    };
    let Some(camera) = brimstone_world
        .resources
        .player
        .camera_entity
        .and_then(|entity| world.core.get_global_transform(entity))
    else {
        return String::new();
    };

    let mut to_target = target - player;
    to_target.y = 0.0;
    let distance = to_target.norm();
    if distance < 0.5 {
        return String::new();
    }
    let direction = to_target / distance;
    let forward = camera.forward_vector();
    let right = camera.right_vector();
    let ahead = direction.x * forward.x + direction.z * forward.z;
    let side = direction.x * right.x + direction.z * right.z;
    let arrow = if ahead < -0.3 {
        "v"
    } else if side > 0.4 {
        ">"
    } else if side < -0.4 {
        "<"
    } else {
        "^"
    };
    format!("    {arrow} {distance:.0}m")
}

fn set_color(world: &mut World, entity: Entity, color: Vec4) {
    if let Some(node_color) = world.ui.get_ui_node_color_mut(entity) {
        node_color.colors[UiBase::INDEX] = Some(color);
    }
}
