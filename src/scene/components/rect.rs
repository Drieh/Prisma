use crate::scene::components::{Style, Transform};
use sdl3::rect::Point;
use sdl3::render::Canvas;
use sdl3::video::Window;
#[derive(Debug, Clone)]
pub struct Rect {
    //transform: Transform,
    //style: Style,
    width: u32,
    height: u32,
}

impl Rect {
    pub fn new() -> Self {
        Self {
            //transform: Transform::new((0, 0), 0.0, (1.0, 1.0)),
            //style: Style::new(Color::RGB(255, 255, 255)),
            width: 0,
            height: 0,
        }
    }

    pub fn size(&mut self, w: u32, h: u32) -> &mut Self {
        self.width = w;
        self.height = h;
        self
    }
    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn draw(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        node_transform: &Transform,
        node_style: &Style,
    ) {
        canvas.set_draw_color(node_style.color);

        let (x, y) = (node_transform.position.x, node_transform.position.y);
        let (scale_x, scale_y) = (node_transform.scale.0, node_transform.scale.1);
        let width = (self.width as f32 * scale_x) as i32;
        let height = (self.height as f32 * scale_y) as i32;

        let rect = sdl3::rect::Rect::new(x as i32, y as i32, width as u32, height as u32);

        canvas.fill_rect(rect).unwrap();

        /*
        let mut points: Vec<Point> = Vec::new();
        points.push(Point::new(0, 0));
        points.push(Point::new(100, 100));

        //self.fill_polygon(points, canvas);

        self.draw_triangle(
            Point::new(0, 0),
            Point::new(100, 100),
            Point::new(200, 0),
            canvas,
        )
         */
    }

    fn fill_polygon(&self, points: Vec<Point>, canvas: &mut Canvas<Window>) {
        let min_x = points.iter().min_by_key(|p| p.x()).unwrap().x();
        let max_x = points.iter().max_by_key(|p| p.x()).unwrap().x();

        for x in min_x..=max_x {
            canvas
                .draw_line(Point::new(x, 0), Point::new(x, 200))
                .expect("Failded to create render");
        }
    }

    fn draw_triangle(&self, p1: Point, p2: Point, p3: Point, canvas: &mut Canvas<Window>) {
        let mut points: Vec<Point> = vec![p1, p2, p3];
        points.sort_by(|p1, p2| p1.x.cmp(&p2.x));

        let point_a = points.get(0).unwrap();
        let point_b = points.get(1).unwrap();
        let point_c = points.get(2).unwrap();

        let line_ab = (point_a, point_b);
        let line_bc = (point_b, point_c);
        let line_ac = (point_a, point_c);

        fn line(x: i32, (p1, p2): (&Point, &Point)) -> i32 {
            let dx = p2.x - p1.x;
            if dx == 0 {
                return p1.y.max(p2.y);
            }
            let m = (p2.y - p1.y) as f32 / dx as f32;
            (m * (x - p1.x) as f32 + p1.y as f32).round() as i32
        }

        for x in point_a.x..=point_c.x {
            let y_ab = line(x, line_ab);
            let y_bc = line(x, line_bc);
            let y_ac = line(x, line_ac);

            let start = Point::new(x, y_ac);
            let end = Point::new(x, y_ab.min(y_bc));

            canvas
                .draw_line(start, end)
                .expect("Failded to create render");
        }
    }
}
