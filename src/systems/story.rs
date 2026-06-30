//! Story-mode director: sequences cutscenes and missions, threading the
//! campaign from the opening transmission through each briefing, mission, and
//! debrief to the ending.

use crate::campaign;
use crate::ecs::{BoomerWorld, Screen, StoryNext, StorySlide};
use crate::systems::lifecycle;
use crate::systems::world::game;
use nightshade::prelude::*;

const PROGRESS_PATH: &str = "boom_campaign.txt";

/// Open the mission picker, loading saved campaign progress on first entry.
pub fn open_select(boomer_world: &mut BoomerWorld, world: &mut World) {
    ensure_loaded(boomer_world);
    boomer_world.resources.story.active = true;
    lifecycle::enter(boomer_world, world, Screen::MissionSelect);
}

/// Begin a mission from the picker: its briefing (and the opening cutscene for
/// mission one) then the mission itself.
pub fn launch_mission(boomer_world: &mut BoomerWorld, world: &mut World, index: usize) {
    ensure_loaded(boomer_world);
    boomer_world.resources.story.active = true;
    boomer_world.resources.story.mission = index;
    let mut slides = if index == 0 {
        intro_slides()
    } else {
        Vec::new()
    };
    slides.push(briefing_slide(index));
    show(boomer_world, world, slides, StoryNext::StartMission(index));
}

fn ensure_loaded(boomer_world: &mut BoomerWorld) {
    if !boomer_world.resources.story.loaded {
        boomer_world.resources.story.unlocked = load_progress();
        boomer_world.resources.story.loaded = true;
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

pub fn mission_complete(boomer_world: &mut BoomerWorld, world: &mut World) {
    let index = boomer_world.resources.story.mission;
    let score = boomer_world.resources.game.score;
    let mut slides = vec![debrief_slide(index, score)];
    let after = if index + 1 < campaign::count() {
        slides.push(briefing_slide(index + 1));
        StoryNext::StartMission(index + 1)
    } else {
        slides.extend(ending_slides());
        StoryNext::Title
    };
    show(boomer_world, world, slides, after);
}

/// Advance the on-screen cutscene; when the slides run out, do the queued action.
pub fn advance(boomer_world: &mut BoomerWorld, world: &mut World) {
    let count = boomer_world.resources.story.slides.len();
    let next_index = boomer_world.resources.story.slide_index + 1;
    if next_index < count {
        boomer_world.resources.story.slide_index = next_index;
        boomer_world.resources.story.reveal = 0.0;
        return;
    }
    match boomer_world.resources.story.after {
        StoryNext::StartMission(index) => start_mission(boomer_world, world, index),
        StoryNext::Title => {
            boomer_world.resources.story.active = false;
            game::start_at(boomer_world, world, 0);
            lifecycle::enter(boomer_world, world, Screen::Title);
        }
    }
}

fn start_mission(boomer_world: &mut BoomerWorld, world: &mut World, index: usize) {
    boomer_world.resources.story.mission = index;
    if index > boomer_world.resources.story.unlocked {
        boomer_world.resources.story.unlocked = index;
        save_progress(index);
    }
    game::start_mission(boomer_world, world, index);
    lifecycle::enter(boomer_world, world, Screen::InGame);
}

fn show(
    boomer_world: &mut BoomerWorld,
    world: &mut World,
    slides: Vec<StorySlide>,
    after: StoryNext,
) {
    boomer_world.resources.story.slides = slides;
    boomer_world.resources.story.slide_index = 0;
    boomer_world.resources.story.reveal = 0.0;
    boomer_world.resources.story.after = after;
    lifecycle::enter(boomer_world, world, Screen::Cutscene);
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
