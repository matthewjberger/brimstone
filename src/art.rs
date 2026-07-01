pub struct Sprite {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const SCALE: u32 = 8;

/// Number of procedurally generated animation frames per enemy.
pub const ANIM_FRAMES: usize = 4;

const IMP: &[&str] = &[
    "...nn........nn.",
    "..nn........nn..",
    "..no........on..",
    "....obbbbbbo....",
    "...obbbbbbbbo...",
    "..obbbbbbbbbbo..",
    "..obeebbbbeebo..",
    "..obeebbbbeebo..",
    "..obbbbmmbbbbo..",
    "..obbbbbbbbbbo..",
    ".obbbbbbbbbbbbo.",
    ".obbbbbbbbbbbbo.",
    ".obbbbbbbbbbbbo.",
    ".obbggggggggbbo.",
    ".obbggggggggbbo.",
    "..obbbbbbbbbbo..",
    "..obbo....obbo..",
    "..obbo....obbo..",
    "..oddo....oddo..",
];

const SWARMER: &[&str] = &[
    "..s........s..",
    "...s......s...",
    "...oggggggo...",
    "..oggggggggo..",
    "..oggeggeggo..",
    "..oggeggeggo..",
    "..oggggggggo..",
    "..oggmmmmggo..",
    ".oggggggggggo.",
    ".oggggggggggo.",
    "..oggggggggo..",
    "s..o.oggo.o..s",
];

const GARGOYLE: &[&str] = &[
    "w.........w",
    "wo.......ow",
    "wwobbbbboww",
    "wwobebeboww",
    "wwobbbbboww",
    ".wobbbbbow.",
    "..obbbbbo..",
    "..obdddbo..",
    "...dd.dd...",
];

// A floating eye: teal orb, scowling brow, a glowing amber iris and a black slit
// pupil. Deliberately no white-on-red, so it reads as a demon eye and not a medkit.
const SENTINEL: &[&str] = &[
    "..bbbbbb...",
    ".bbbbbbbbb.",
    "ottiiiiitto",
    "otiiiiiiito",
    "otiiipiiito",
    "otiiipiiito",
    "otiiipiiito",
    "otiiiiiiito",
    "ottiiiiitto",
    ".ottttttto.",
    "..ooooooo..",
];

const CASTER: &[&str] = &[
    "....oooo....",
    "...orrrro...",
    "..orrrrrro..",
    "..orrrrrro..",
    "..orreerro..",
    "..orreerro..",
    "..orrrrrro..",
    ".orrrrrrrro.",
    ".orrrrrrrro.",
    ".orrddddrro.",
    ".orrddddrro.",
    ".orrddddrro.",
    ".orrddddrro.",
    "..orddddro..",
    "..orddddro..",
    "...orddro...",
    "....oddo....",
    "....o..o....",
];

const BRUTE: &[&str] = &[
    "....hh......hh....",
    "...haa......aah...",
    "...haaa....aaah...",
    "....oaaaaaaaao....",
    "...oaaaaaaaaaao...",
    "..oaaaaaaaaaaaao..",
    "..oaaeeaaaaeeaao..",
    "..oaaeeaaaaeeaao..",
    "..oaaaaammaaaaao..",
    "..oaaaammmmaaaao..",
    ".oaaaaaaaaaaaaaao.",
    "oaaaaaaaaaaaaaaaao",
    "oaaaddaaaaaaddaaao",
    "oaaaddaaaaaaddaaao",
    "oaaaaaaaaaaaaaaaao",
    ".oaaaaaaaaaaaaaao.",
    "..oaao......oaao..",
    "..oaao......oaao..",
    "..oddo......oddo..",
    "..oddo......oddo..",
];

fn brute_color(symbol: char, hurt: bool) -> [u8; 4] {
    let base = match symbol {
        'o' => [26, 8, 4, 255],
        'a' => [196, 70, 22, 255],
        'd' => [96, 30, 10, 255],
        'm' => [40, 16, 10, 255],
        'e' => [255, 232, 90, 255],
        'h' => [60, 18, 8, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

pub fn brute_idle() -> Sprite {
    render_grid(BRUTE, |symbol| brute_color(symbol, false))
}

pub fn brute_hurt() -> Sprite {
    render_grid(BRUTE, |symbol| brute_color(symbol, true))
}

fn brighten(color: [u8; 4], hurt: bool) -> [u8; 4] {
    if !hurt || color[3] == 0 {
        return color;
    }
    [
        (color[0] as u16 + (255 - color[0] as u16) * 3 / 4) as u8,
        (color[1] as u16 + (255 - color[1] as u16) * 3 / 4) as u8,
        (color[2] as u16 + (255 - color[2] as u16) * 3 / 4) as u8,
        255,
    ]
}

fn imp_color(symbol: char, hurt: bool) -> [u8; 4] {
    let base = match symbol {
        'o' => [22, 8, 8, 255],
        'b' => [156, 30, 30, 255],
        'd' => [70, 14, 14, 255],
        'g' => [112, 36, 36, 255],
        'e' => [255, 224, 70, 255],
        'm' => [240, 240, 232, 255],
        'n' => [205, 192, 158, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

fn swarmer_color(symbol: char, hurt: bool) -> [u8; 4] {
    let base = match symbol {
        'o' => [10, 30, 15, 255],
        'g' => [46, 156, 64, 255],
        'd' => [22, 92, 36, 255],
        's' => [128, 206, 86, 255],
        'e' => [255, 72, 52, 255],
        'm' => [236, 240, 224, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

fn gargoyle_color(symbol: char, hurt: bool) -> [u8; 4] {
    let base = match symbol {
        'w' => [92, 100, 126, 255],
        'o' => [16, 14, 28, 255],
        'b' => [124, 82, 184, 255],
        'd' => [58, 38, 96, 255],
        'e' => [140, 255, 240, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

pub fn gargoyle_idle() -> Sprite {
    render_grid(GARGOYLE, |symbol| gargoyle_color(symbol, false))
}

pub fn gargoyle_hurt() -> Sprite {
    render_grid(GARGOYLE, |symbol| gargoyle_color(symbol, true))
}

fn sentinel_color(symbol: char, hurt: bool) -> [u8; 4] {
    let base = match symbol {
        'o' => [10, 28, 32, 255],
        'b' => [6, 14, 16, 255],
        't' => [40, 150, 160, 255],
        'i' => [255, 150, 30, 255],
        'p' => [10, 6, 4, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

pub fn sentinel_idle() -> Sprite {
    render_grid(SENTINEL, |symbol| sentinel_color(symbol, false))
}

pub fn sentinel_hurt() -> Sprite {
    render_grid(SENTINEL, |symbol| sentinel_color(symbol, true))
}

fn caster_color(symbol: char, hurt: bool) -> [u8; 4] {
    let base = match symbol {
        'o' => [20, 8, 35, 255],
        'r' => [120, 54, 170, 255],
        'd' => [64, 26, 100, 255],
        'e' => [130, 255, 255, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

fn render_grid(grid: &[&str], color: impl Fn(char) -> [u8; 4]) -> Sprite {
    let rows = grid.len() as u32;
    let cols = grid
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0) as u32;
    let width = cols * SCALE;
    let height = rows * SCALE;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for (row_index, row) in grid.iter().enumerate() {
        for (col_index, symbol) in row.chars().enumerate() {
            let pixel = color(symbol);
            blit_cell(&mut rgba, width, col_index as u32, row_index as u32, pixel);
        }
    }
    Sprite {
        rgba,
        width,
        height,
    }
}

fn blit_cell(rgba: &mut [u8], width: u32, cell_x: u32, cell_y: u32, pixel: [u8; 4]) {
    for y in 0..SCALE {
        for x in 0..SCALE {
            let px = cell_x * SCALE + x;
            let py = cell_y * SCALE + y;
            let index = ((py * width + px) * 4) as usize;
            rgba[index] = pixel[0];
            rgba[index + 1] = pixel[1];
            rgba[index + 2] = pixel[2];
            rgba[index + 3] = pixel[3];
        }
    }
}

/// Rotate a sprite by `radians` (nearest-neighbour) into a padded canvas, so the
/// corners don't clip. Used to make the angled hip-fire viewmodel pose from the
/// upright aim-down-sights sprite, guaranteeing both poses are the same weapon.
pub fn tilted(base: &Sprite, radians: f32) -> Sprite {
    const PAD: u32 = 44;
    let width = base.width + PAD * 2;
    let height = base.height + PAD * 2;
    let mut out = solid(width, height);
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let base_center_x = base.width as f32 / 2.0;
    let base_center_y = base.height as f32 / 2.0;
    let (sin, cos) = radians.sin_cos();
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;
            let source_x = cos * dx + sin * dy + base_center_x;
            let source_y = -sin * dx + cos * dy + base_center_y;
            if source_x >= 0.0
                && source_y >= 0.0
                && (source_x as u32) < base.width
                && (source_y as u32) < base.height
            {
                let source = (((source_y as u32) * base.width + (source_x as u32)) * 4) as usize;
                let dest = ((y * width + x) * 4) as usize;
                out.rgba[dest..dest + 4].copy_from_slice(&base.rgba[source..source + 4]);
            }
        }
    }
    out
}

/// Shift a sprite by whole pixels, leaving exposed edges transparent. Used to
/// synthesize animation frames (a bob / sway cycle) from a single base sprite.
fn offset(base: &Sprite, dx: i32, dy: i32) -> Sprite {
    let width = base.width as i32;
    let height = base.height as i32;
    let mut out = solid(base.width, base.height);
    for y in 0..height {
        for x in 0..width {
            let sx = x - dx;
            let sy = y - dy;
            if sx >= 0 && sx < width && sy >= 0 && sy < height {
                let source = ((sy * width + sx) * 4) as usize;
                let dest = ((y * width + x) * 4) as usize;
                out.rgba[dest..dest + 4].copy_from_slice(&base.rgba[source..source + 4]);
            }
        }
    }
    out
}

/// Produce animation frame `index` from a base sprite: a small up-bob with a
/// left/right sway that reads as breathing when idle and a stride when moving.
pub fn frame(base: &Sprite, index: usize) -> Sprite {
    // A fuller bob-and-sway cycle (rise, peak, fall, settle) at a bigger amplitude
    // than before, so idle enemies visibly breathe and moving ones bounce.
    const TABLE: [(i32, i32); ANIM_FRAMES] = [(0, 0), (2, -5), (0, -9), (-2, -5)];
    let (dx, dy) = TABLE[index % ANIM_FRAMES];
    offset(base, dx, dy)
}

pub fn imp_idle() -> Sprite {
    render_grid(IMP, |symbol| imp_color(symbol, false))
}

pub fn imp_hurt() -> Sprite {
    render_grid(IMP, |symbol| imp_color(symbol, true))
}

pub fn swarmer_idle() -> Sprite {
    render_grid(SWARMER, |symbol| swarmer_color(symbol, false))
}

pub fn swarmer_hurt() -> Sprite {
    render_grid(SWARMER, |symbol| swarmer_color(symbol, true))
}

pub fn caster_idle() -> Sprite {
    render_grid(CASTER, |symbol| caster_color(symbol, false))
}

pub fn caster_hurt() -> Sprite {
    render_grid(CASTER, |symbol| caster_color(symbol, true))
}

fn solid(width: u32, height: u32) -> Sprite {
    Sprite {
        rgba: vec![0u8; (width * height * 4) as usize],
        width,
        height,
    }
}

fn put(sprite: &mut Sprite, x: u32, y: u32, pixel: [u8; 4]) {
    if x >= sprite.width || y >= sprite.height {
        return;
    }
    let index = ((y * sprite.width + x) * 4) as usize;
    sprite.rgba[index] = pixel[0];
    sprite.rgba[index + 1] = pixel[1];
    sprite.rgba[index + 2] = pixel[2];
    sprite.rgba[index + 3] = pixel[3];
}

// Pickup items, hand-drawn in the same grid style as the enemies.
fn item_color(symbol: char) -> [u8; 4] {
    match symbol {
        'o' => [24, 20, 14, 255],
        'w' => [236, 240, 236, 255],
        'r' => [220, 44, 40, 255],
        'm' => [64, 58, 44, 255],
        'a' => [240, 196, 72, 255],
        'b' => [250, 214, 92, 255],
        'c' => [230, 196, 40, 255],
        's' => [40, 32, 12, 255],
        'h' => [250, 240, 180, 255],
        _ => [0, 0, 0, 0],
    }
}

const MEDKIT: &[&str] = &[
    "...ooooooo...",
    ".ooooooooooo.",
    ".owwwwwwwwwo.",
    ".owwwwrwwwwo.",
    ".owwwwrwwwwo.",
    ".owwrrrrrwwo.",
    ".owwrrrrrwwo.",
    ".owwwwrwwwwo.",
    ".owwwwrwwwwo.",
    ".owwwwwwwwwo.",
    ".ooooooooooo.",
];

const AMMO_BOX: &[&str] = &[
    "..b.b.b.b.b..",
    "..m.m.m.m.m..",
    ".ooooooooooo.",
    ".ommmmmmmmmo.",
    ".oaaaaaaaaao.",
    ".ommmmmmmmmo.",
    ".ommmmmmmmmo.",
    ".ooooooooooo.",
];

const KEYCARD: &[&str] = &[
    ".ooooooooooo.",
    ".occccccccco.",
    ".ossssssssso.",
    ".occccccccco.",
    ".occhhccccco.",
    ".occhhccccco.",
    ".occccccccco.",
    ".ooooooooooo.",
];

pub fn medkit() -> Sprite {
    render_grid(MEDKIT, item_color)
}

pub fn ammo_box() -> Sprite {
    render_grid(AMMO_BOX, item_color)
}

fn radial(size: u32, inner: [u8; 3], outer: [u8; 3]) -> Sprite {
    let mut sprite = solid(size, size);
    let center = size as f32 / 2.0;
    let radius = center;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt() / radius;
            if distance >= 1.0 {
                continue;
            }
            let falloff = 1.0 - distance;
            let alpha = (falloff.powf(1.4) * 255.0) as u8;
            let blend = distance;
            let red = (inner[0] as f32 * (1.0 - blend) + outer[0] as f32 * blend) as u8;
            let green = (inner[1] as f32 * (1.0 - blend) + outer[1] as f32 * blend) as u8;
            let blue = (inner[2] as f32 * (1.0 - blend) + outer[2] as f32 * blend) as u8;
            put(&mut sprite, x, y, [red, green, blue, alpha]);
        }
    }
    sprite
}

pub fn fireball() -> Sprite {
    radial(96, [255, 248, 210], [240, 70, 18])
}

pub fn rocket() -> Sprite {
    radial(96, [236, 252, 255], [70, 150, 255])
}

// A simple humanoid townsperson; colour the robe/accent/hair to make distinct
// adventure-mode NPCs from one silhouette.
const PERSON: &[&str] = &[
    "...kkk...",
    "..kkkkk..",
    "..ksssk..",
    "..sbsbs..",
    "..ksssk..",
    "...aaa...",
    "..raaar..",
    ".rraaarr.",
    ".rraaarr.",
    ".rrrrrrr.",
    "..rr.rr..",
    "..dd.dd..",
];

fn person_color(
    symbol: char,
    robe: [u8; 4],
    accent: [u8; 4],
    hair: [u8; 4],
    hurt: bool,
) -> [u8; 4] {
    let base = match symbol {
        'k' => hair,
        's' => [226, 178, 140, 255],
        'b' => [40, 26, 22, 255],
        'a' => accent,
        'r' => robe,
        'd' => [42, 32, 26, 255],
        _ => [0, 0, 0, 0],
    };
    brighten(base, hurt)
}

fn person(robe: [u8; 4], accent: [u8; 4], hair: [u8; 4]) -> Sprite {
    render_grid(PERSON, |symbol| {
        person_color(symbol, robe, accent, hair, false)
    })
}

pub fn npc_villager() -> Sprite {
    person([120, 84, 52, 255], [92, 62, 40, 255], [60, 40, 28, 255])
}

pub fn npc_merchant() -> Sprite {
    person([96, 52, 140, 255], [232, 196, 72, 255], [40, 30, 28, 255])
}

pub fn npc_elder() -> Sprite {
    person(
        [176, 178, 190, 255],
        [120, 124, 144, 255],
        [224, 224, 230, 255],
    )
}

pub fn npc_guard() -> Sprite {
    person([78, 96, 122, 255], [156, 168, 188, 255], [50, 44, 40, 255])
}

// First-person weapon viewmodels: a bottom-anchored pixel sprite of
// the held weapon, drawn as a screen overlay. Same hand-drawn grid style as the
// enemies (16 cols wide), muzzle at the top, gripping hands at the bottom.
fn viewmodel_color(symbol: char, accent: [u8; 4]) -> [u8; 4] {
    match symbol {
        'o' => [30, 32, 38, 255],
        'm' => [96, 100, 112, 255],
        'l' => [165, 170, 182, 255],
        'a' => accent,
        'g' => [58, 42, 30, 255],
        'h' => [226, 178, 140, 255],
        'k' => [200, 150, 116, 255],
        _ => [0, 0, 0, 0],
    }
}

fn weapon_view(grid: &[&str], accent: [u8; 4]) -> Sprite {
    render_grid(grid, |symbol| viewmodel_color(symbol, accent))
}

const VM_SHOTGUN: &[&str] = &[
    "....mm......mm....",
    "....oo......oo....",
    "...ommo....ommo...",
    "..ommmmoooommmmo..",
    ".ommmmmmmmmmmmmmo.",
    ".ommaaaaaaaaaammo.",
    ".olmmmmmmmmmmmmlo.",
    ".ommmmmmmmmmmmmmo.",
    ".ohhhhhhhhhhhhhho.",
    ".okhhhhhhhhhhhhko.",
    "...ommgggggmmo....",
    "....ogggggggo.....",
    "....ogggggggo.....",
    ".....ggggggg......",
];

const VM_NAILGUN: &[&str] = &[
    "...m..m..m..m.....",
    "...m..m..m..m.....",
    "..oooooooooooooo..",
    ".ommmmmmmmmmmmmmo.",
    ".ommaaaaaaaaaammo.",
    ".olmmmmmmmmmmmmlo.",
    ".ommmmmmmmmmmmmmo.",
    ".ohhhhhhhhhhhhhho.",
    ".okhhhhhhhhhhhhko.",
    "...ommgggggmmo....",
    "....ogggggggo.....",
    "....ogggggggo.....",
    ".....ggggggg......",
];

const VM_ROCKET: &[&str] = &[
    "....oooooooooo....",
    "...ommaaaaaammo...",
    "...ommmmmmmmmmo...",
    "..ommmmmmmmmmmmo..",
    ".olmmmmmmmmmmmmlo.",
    ".ommmmmmmmmmmmmmo.",
    ".ohhhhhhhhhhhhhho.",
    ".okhhhhhhhhhhhhko.",
    "...ommgggggmmo....",
    "....ogggggggo.....",
    "....ogggggggo.....",
    ".....ggggggg......",
];

const VM_RAILGUN: &[&str] = &[
    "....aaaaaaaaaa....",
    "...ommmmmmmmmmo...",
    "..ommmmmmmmmmmmo..",
    ".ommaaaaaaaaaammo.",
    ".olmmmmmmmmmmmmlo.",
    ".ommaaaaaaaaaammo.",
    ".ommmmmmmmmmmmmmo.",
    ".ohhhhhhhhhhhhhho.",
    ".okhhhhhhhhhhhhko.",
    "...ommgggggmmo....",
    "....ogggggggo.....",
    "....ogggggggo.....",
    ".....ggggggg......",
];

const VM_PISTOL: &[&str] = &[
    ".......oooo.......",
    ".......mllm.......",
    ".....ommmmmmo.....",
    ".....ommaammo.....",
    ".....ommmmmmo.....",
    "....ohhhhhhhho....",
    "....okhhhhhhko....",
    ".....oggggggo.....",
    ".....oggggggo.....",
    ".....oggggggo.....",
    "......gggggg......",
];

pub fn viewmodel_shotgun() -> Sprite {
    weapon_view(VM_SHOTGUN, [240, 150, 60, 255])
}

pub fn viewmodel_nailgun() -> Sprite {
    weapon_view(VM_NAILGUN, [70, 205, 232, 255])
}

pub fn viewmodel_rocket() -> Sprite {
    weapon_view(VM_ROCKET, [96, 156, 255, 255])
}

pub fn viewmodel_railgun() -> Sprite {
    weapon_view(VM_RAILGUN, [176, 96, 240, 255])
}

pub fn viewmodel_pistol() -> Sprite {
    weapon_view(VM_PISTOL, [240, 210, 90, 255])
}

pub fn keycard() -> Sprite {
    render_grid(KEYCARD, item_color)
}
