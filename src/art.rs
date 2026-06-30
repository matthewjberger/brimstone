pub struct Sprite {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const SCALE: u32 = 8;

/// Number of procedurally generated animation frames per enemy.
pub const ANIM_FRAMES: usize = 3;

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
    let cols = grid[0].len() as u32;
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
    const TABLE: [(i32, i32); ANIM_FRAMES] = [(0, 0), (1, -3), (-1, -1)];
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

fn fill_rect(sprite: &mut Sprite, x0: u32, y0: u32, x1: u32, y1: u32, pixel: [u8; 4]) {
    for y in y0..y1 {
        for x in x0..x1 {
            put(sprite, x, y, pixel);
        }
    }
}

pub fn medkit() -> Sprite {
    let size = 16 * SCALE;
    let mut sprite = solid(size, size);
    let outline = [24, 18, 14, 255];
    let white = [236, 240, 236, 255];
    let red = [228, 44, 40, 255];
    fill_rect(
        &mut sprite,
        SCALE,
        2 * SCALE,
        15 * SCALE,
        15 * SCALE,
        outline,
    );
    fill_rect(
        &mut sprite,
        2 * SCALE,
        3 * SCALE,
        14 * SCALE,
        14 * SCALE,
        white,
    );
    fill_rect(
        &mut sprite,
        7 * SCALE,
        5 * SCALE,
        9 * SCALE,
        12 * SCALE,
        red,
    );
    fill_rect(
        &mut sprite,
        5 * SCALE,
        7 * SCALE,
        11 * SCALE,
        10 * SCALE,
        red,
    );
    sprite
}

pub fn ammo_box() -> Sprite {
    let size = 16 * SCALE;
    let mut sprite = solid(size, size);
    let outline = [22, 20, 12, 255];
    let shell = [40, 36, 30, 255];
    let brass = [240, 196, 72, 255];
    fill_rect(
        &mut sprite,
        2 * SCALE,
        4 * SCALE,
        14 * SCALE,
        14 * SCALE,
        outline,
    );
    fill_rect(
        &mut sprite,
        3 * SCALE,
        5 * SCALE,
        13 * SCALE,
        13 * SCALE,
        shell,
    );
    for slot in 0..4 {
        let x = (4 + slot * 2) * SCALE;
        fill_rect(&mut sprite, x, 5 * SCALE, x + SCALE, 9 * SCALE, brass);
    }
    sprite
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

pub fn keycard() -> Sprite {
    let size = 16 * SCALE;
    let mut sprite = solid(size, size);
    let outline = [18, 20, 12, 255];
    let card = [230, 196, 40, 255];
    let stripe = [40, 32, 12, 255];
    let chip = [250, 240, 180, 255];
    fill_rect(
        &mut sprite,
        3 * SCALE,
        5 * SCALE,
        13 * SCALE,
        12 * SCALE,
        outline,
    );
    fill_rect(
        &mut sprite,
        4 * SCALE,
        6 * SCALE,
        12 * SCALE,
        11 * SCALE,
        card,
    );
    fill_rect(
        &mut sprite,
        4 * SCALE,
        7 * SCALE,
        12 * SCALE,
        8 * SCALE,
        stripe,
    );
    fill_rect(
        &mut sprite,
        5 * SCALE,
        9 * SCALE,
        7 * SCALE,
        10 * SCALE,
        chip,
    );
    sprite
}
