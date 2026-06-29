pub struct Sprite {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const SCALE: u32 = 8;

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

fn imp_palette(symbol: char, body: [u8; 4], eye: [u8; 4], belly: [u8; 4]) -> [u8; 4] {
    match symbol {
        'o' => [22, 8, 8, 255],
        'b' => body,
        'd' => [70, 14, 14, 255],
        'g' => belly,
        'e' => eye,
        'm' => [240, 240, 232, 255],
        'n' => [205, 192, 158, 255],
        _ => [0, 0, 0, 0],
    }
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

pub fn imp_idle() -> Sprite {
    render_grid(IMP, |symbol| {
        imp_palette(
            symbol,
            [150, 28, 28, 255],
            [255, 224, 70, 255],
            [110, 36, 36, 255],
        )
    })
}

pub fn imp_attack() -> Sprite {
    render_grid(IMP, |symbol| {
        imp_palette(
            symbol,
            [206, 64, 30, 255],
            [255, 250, 210, 255],
            [168, 54, 22, 255],
        )
    })
}

pub fn imp_hurt() -> Sprite {
    render_grid(IMP, |symbol| {
        imp_palette(
            symbol,
            [232, 196, 196, 255],
            [255, 255, 255, 255],
            [220, 180, 180, 255],
        )
    })
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
    let white = [236, 236, 230, 255];
    let red = [206, 40, 36, 255];
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
    let brass = [216, 176, 64, 255];
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
            let alpha = (falloff * falloff * 255.0) as u8;
            let blend = distance;
            let red = (inner[0] as f32 * (1.0 - blend) + outer[0] as f32 * blend) as u8;
            let green = (inner[1] as f32 * (1.0 - blend) + outer[1] as f32 * blend) as u8;
            let blue = (inner[2] as f32 * (1.0 - blend) + outer[2] as f32 * blend) as u8;
            put(&mut sprite, x, y, [red, green, blue, alpha]);
        }
    }
    sprite
}

pub fn muzzle() -> Sprite {
    radial(96, [255, 248, 200], [240, 150, 30])
}

pub fn spark() -> Sprite {
    radial(64, [255, 230, 180], [220, 70, 20])
}
