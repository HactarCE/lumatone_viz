use std::sync::LazyLock;

use egui::{Vec2, vec2};

pub const INRADIUS: f32 = 1.0;
pub const W: f32 = INRADIUS;
pub static SQRT_3_2: LazyLock<f32> = LazyLock::new(|| 3.0_f32.sqrt() / 2.0);
pub static H: LazyLock<f32> = LazyLock::new(|| INRADIUS / *SQRT_3_2);

pub const DX: f32 = W;
pub static DY: LazyLock<f32> = LazyLock::new(|| *H * 0.75);
pub static DELTA: LazyLock<Vec2> = LazyLock::new(|| vec2(DX, *DY));

pub const ROW_LENGTHS: &[usize] = &[2, 5, 6, 6, 6, 6, 6, 6, 6, 5, 2];
pub const ROW_OFFSETS: &[usize] = &[0, 1, 0, 1, 0, 1, 0, 1, 0, 3, 8];
pub const BOARD_OFFSET: Vec2 = vec2(6.0, 2.0);
pub const BOARD_CENTER: Vec2 = vec2(3.0, 4.5);
pub const TOP_LEFT_PAD: Vec2 = vec2(0.5, -0.5);
pub static TOTAL_SIZE: LazyLock<Vec2> = LazyLock::new(|| {
    rotate(
        TOP_LEFT_PAD + BOARD_CENTER * 2.0 + BOARD_OFFSET * 4.0,
        *ANGLE,
    )
});

pub static ANGLE: LazyLock<f32> = LazyLock::new(|| (BOARD_OFFSET * *DELTA).angle());

pub static REGULAR_HEXAGON: LazyLock<[Vec2; 6]> = LazyLock::new(|| {
    let w = W;
    let h = *H;

    let w1 = w / 2.0;
    let h2 = h / 2.0;
    let h1 = h / 4.0;

    [
        vec2(w1, h1),
        vec2(0.0, h2),
        vec2(-w1, h1),
        vec2(-w1, -h1),
        vec2(0.0, -h2),
        vec2(w1, -h1),
    ]
});

pub fn hexagon_coordinates(board: usize, mut index: usize) -> [Vec2; 6] {
    let mut x = 0.0;
    let mut y = 0.0;
    for (&len, &offset) in std::iter::zip(ROW_LENGTHS, ROW_OFFSETS) {
        x = offset as f32 / 2.0 + index as f32;
        if index >= len {
            index -= len;
            y += 1.0;
        } else {
            break;
        }
    }

    let center = BOARD_CENTER + 2.0 * BOARD_OFFSET;

    let base = (BOARD_OFFSET * board as f32 + vec2(x, y) - center) * *DELTA;
    REGULAR_HEXAGON.map(|v| rotate(base + v, *ANGLE))
}

pub fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    vec2(cos * v.x + sin * v.y, cos * v.y - sin * v.x)
}
