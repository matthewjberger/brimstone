/// Frame-rate independent exponential approach toward a target.
pub fn approach(current: f32, target: f32, rate: f32) -> f32 {
    current + (target - current) * rate.clamp(0.0, 1.0)
}

/// xorshift64 returning a float in [0, 1). Seeded from uptime once per run.
pub fn next_random(state: &mut u64) -> f32 {
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    *state = value;
    ((value >> 40) as f32) / ((1u64 << 24) as f32)
}

pub fn random_range(state: &mut u64, low: f32, high: f32) -> f32 {
    low + (high - low) * next_random(state)
}

pub fn combo_multiplier(combo: u32) -> u32 {
    1 + (combo / 8).min(4)
}
