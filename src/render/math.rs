use sdl3::rect::Point;

use crate::util::Position;

pub fn line(x: i32, (p1, p2): (&Point, &Point)) -> i32 {
    let dx = p2.x - p1.x;
    if dx == 0 {
        return p1.y.max(p2.y);
    }
    let m = (p2.y - p1.y) as i32 / dx as i32;
    m * (x - p1.x) as i32 + p1.y as i32
}

/// Does not include center.y
pub fn elipse_top_arc(x: i32, radius_x: i32, radius_y: i32, center: Position) -> i32 {
    let x = x as f32 - center.x as f32;
    let a = radius_x as f32;
    let b = radius_y as f32;
    let y = (1.0 - x * x / (a * a)).max(0.0);
    ((y as f32).sqrt() * b).round() as i32
}
