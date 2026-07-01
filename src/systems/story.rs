//! Story-mode director: sequences cutscenes and missions, threading the
//! campaign from the opening transmission through each briefing, mission, and
//! debrief to the ending.

use crate::campaign;
use crate::ecs::{BrimstoneWorld, Screen, StoryNext, StorySlide};
use crate::systems::lifecycle;
use crate::systems::world::game;
use nightshade::prelude::*;

const PROGRESS_PATH: &str = "brimstone_campaign.txt";

/// Open the mission picker, loading saved campaign progress on first entry.
pub fn open_select(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    ensure_loaded(brimstone_world);
    brimstone_world.resources.story.active = true;
    lifecycle::enter(brimstone_world, world, Screen::MissionSelect);
}

/// Begin a mission from the picker: its briefing (and the opening cutscene for
/// mission one) then the mission itself.
pub fn launch_mission(brimstone_world: &mut BrimstoneWorld, world: &mut World, index: usize) {
    ensure_loaded(brimstone_world);
    brimstone_world.resources.story.active = true;
    brimstone_world.resources.story.mission = index;
    let mut slides = if index == 0 {
        intro_slides()
    } else {
        Vec::new()
    };
    slides.push(briefing_slide(index));
    show(brimstone_world, world, slides, StoryNext::StartMission(index));
}

fn ensure_loaded(brimstone_world: &mut BrimstoneWorld) {
    if !brimstone_world.resources.story.loaded {
        brimstone_world.resources.story.unlocked = load_progress();
        brimstone_world.resources.story.loaded = true;
    }
}

fn load_progress() -> usize {
    std::fs::read_to_string(PROGRESS_PATH)
        .ok()
        .and_then(|text| text.trim().parse::<usize>().ok())
        .unwrap_or(0)
        .min(campaign::count().saturating_sub(1))
}

fn save_progress(value: usize) {
    let _ = std::fs::write(PROGRESS_PATH, value.to_string());
}

pub fn mission_complete(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let index = brimstone_world.resources.story.mission;
    let score = brimstone_world.resources.game.score;
    let mut slides = vec![debrief_slide(index, score)];
    let after = if index + 1 < campaign::count() {
        slides.push(briefing_slide(index + 1));
        StoryNext::StartMission(index + 1)
    } else {
        slides.extend(ending_slides());
        StoryNext::Title
    };
    show(brimstone_world, world, slides, after);
}

/// Advance the on-screen cutscene; when the slides run out, do the queued action.
pub fn advance(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let count = brimstone_world.resources.story.slides.len();
    let next_index = brimstone_world.resources.story.slide_index + 1;
    if next_index < count {
        brimstone_world.resources.story.slide_index = next_index;
        brimstone_world.resources.story.reveal = 0.0;
        return;
    }
    match brimstone_world.resources.story.after {
        StoryNext::StartMission(index) => start_mission(brimstone_world, world, index),
        StoryNext::Title => {
            brimstone_world.resources.story.active = false;
            game::start_at(brimstone_world, world, 0);
            lifecycle::enter(brimstone_world, world, Screen::Title);
        }
    }
}

fn start_mission(brimstone_world: &mut BrimstoneWorld, world: &mut World, index: usize) {
    brimstone_world.resources.story.mission = index;
    if index > brimstone_world.resources.story.unlocked {
        brimstone_world.resources.story.unlocked = index;
        save_progress(index);
    }
    game::start_mission(brimstone_world, world, index);
    lifecycle::enter(brimstone_world, world, Screen::InGame);
}

fn show(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    slides: Vec<StorySlide>,
    after: StoryNext,
) {
    brimstone_world.resources.story.slides = slides;
    brimstone_world.resources.story.slide_index = 0;
    brimstone_world.resources.story.reveal = 0.0;
    brimstone_world.resources.story.after = after;
    lifecycle::enter(brimstone_world, world, Screen::Cutscene);
}

fn slide(title: impl Into<String>, body: impl Into<String>) -> StorySlide {
    StorySlide {
        title: title.into(),
        body: body.into(),
    }
}

fn intro_slides() -> Vec<StorySlide> {
    campaign::INTRO
        .iter()
        .map(|body| slide("INCOMING TRANSMISSION", *body))
        .collect()
}

fn ending_slides() -> Vec<StorySlide> {
    campaign::ENDING
        .iter()
        .map(|body| slide("GEHENNA", *body))
        .collect()
}

fn briefing_slide(index: usize) -> StorySlide {
    let mission = campaign::mission(index);
    slide(
        format!("MISSION {}: {}", index + 1, mission.title),
        format!(
            "OBJECTIVE — {}\n\n{}",
            mission.objective.label(),
            mission.briefing
        ),
    )
}

fn debrief_slide(index: usize, score: u32) -> StorySlide {
    let mission = campaign::mission(index);
    slide(
        "MISSION COMPLETE",
        format!("{}\n\nSCORE  {score}", mission.debrief),
    )
}
