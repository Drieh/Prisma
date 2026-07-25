use std::collections::BTreeMap;

use crate::{
    scene::{NodeID, Scene, components::Transform},
    util::Position,
};
use sdl3::{pixels::Color as SdlColor, rect::Point, render::Canvas, video::Window};

pub struct Renderer {
    canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new(canvas: Canvas<Window>) -> Self {
        let mut _self = Self { canvas };
        _self.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        _self
    }

    pub fn draw(&mut self, scene: &mut Scene) {
        let render_queue = self.build_render_layers(scene);

        self.render(render_queue, scene);
        self.canvas.present();
    }

    fn build_render_layers(&mut self, scene: &mut Scene) -> BTreeMap<usize, Vec<NodeID>> {
        let mut render_queue: BTreeMap<usize, Vec<NodeID>> = BTreeMap::new();

        for id in scene.get_nodes_id() {
            let node = scene.get_node(id).unwrap();
            if node.get_parent().is_none() {
                let layer = node.get_transform().clone().layer;
                self.visit(id, scene, layer, &mut render_queue);
            }
        }
        render_queue
    }
    fn visit(
        &self,
        node_id: NodeID,
        scene: &mut Scene,
        parent_layer: Option<usize>,
        render_queue: &mut BTreeMap<usize, Vec<NodeID>>,
    ) {
        let node = scene.get_node(node_id).unwrap();

        let layer = node.get_transform().layer.or(parent_layer).unwrap_or(0);

        render_queue.entry(layer).or_default().push(node_id);

        for child in node.get_children() {
            self.visit(child, scene, Some(layer), render_queue);
        }
    }
    fn render(&mut self, render_queue: BTreeMap<usize, Vec<NodeID>>, scene: &mut Scene) {
        self.canvas.set_draw_color(scene.color);
        self.canvas.clear();

        for layer in render_queue.values() {
            for node_id in layer {
                self.render_node(*node_id, scene);
            }
        }
    }
    fn render_node(&mut self, id: NodeID, scene: &mut Scene) {
        let world_position = scene.get_node(id).unwrap().get_world_position();
        let node = scene
            .get_node(id)
            .expect("Internal invariant violated: render layer contains an invalid node ID");

        let node_transform = node.get_transform().clone();

        let draw_transform = Transform {
            position: world_position,
            ..node_transform
        };
        let color = node.get_style().color;
        self.canvas.set_draw_color(SdlColor {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        });

        let (width, height) = node.get_bouding_box_size();
        let position = draw_transform.position;
        let border_radius = node.get_style().border_radius;
        let (scale_x, scale_y) = (draw_transform.scale.0, draw_transform.scale.1);

        let radius_x = if border_radius as f32 * scale_x as f32 > width as f32 / 2.0 {
            (width / 2) as u32
        } else {
            (border_radius as f32 * scale_x).round() as u32
        };

        let radius_y = if border_radius as f32 * scale_y as f32 > height as f32 / 2.0 {
            (height / 2) as u32
        } else {
            (border_radius as f32 * scale_y).round() as u32
        };

        for x in (position.x.round() as i32)..=(position.x as i32 + width as i32) {
            let mut start = Point::new(x, 0);
            let mut end = Point::new(x, 0);

            // Esquinas izquierdas
            if x as f32 <= position.x + radius_x as f32 {
                let top_center = Position {
                    x: position.x + radius_x as f32,
                    y: position.y + radius_y as f32,
                };
                let bottom_center = Position {
                    x: position.x + radius_x as f32,
                    y: position.y - radius_y as f32 + height as f32,
                };
                start.y = -self.elipse_top_arc(x, radius_x, radius_y, top_center)
                    + position.y as i32
                    + radius_y as i32;

                end.y = self.elipse_top_arc(x, radius_x, radius_y, bottom_center)
                    + position.y as i32
                    - radius_y as i32
                    + height as i32;
            }
            // Esquinas derechas
            else if x as f32 >= position.x + width as f32 - radius_x as f32 {
                let center = Position {
                    x: position.x + width as f32 - radius_x as f32,
                    y: position.y + radius_y as f32,
                };

                start.y = -self.elipse_top_arc(x, radius_x, radius_y, center)
                    + position.y as i32
                    + radius_y as i32;

                end.y = self.elipse_top_arc(x, radius_x, radius_y, center)
                    + position.y as i32
                    + height as i32
                    - radius_y as i32;
            }
            // Parte recta
            else {
                start.y = position.y.round() as i32;
                end.y = position.y.round() as i32 + height as i32;
            }

            self.canvas
                .draw_line(start, end)
                .expect("Failed to draw line");
        }
    }

    fn draw_triangle(&mut self, p1: Point, p2: Point, p3: Point) {
        let mut points: Vec<Point> = vec![p1, p2, p3];
        points.sort_by(|p1, p2| p1.x.cmp(&p2.x));

        let point_a = points.get(0).unwrap();
        let point_b = points.get(1).unwrap();
        let point_c = points.get(2).unwrap();

        let line_ab = (point_a, point_b);
        let line_bc = (point_b, point_c);
        let line_ac = (point_a, point_c);

        for x in point_a.x..=point_c.x {
            let y_ab = self.line(x, line_ab);
            let y_bc = self.line(x, line_bc);
            let y_ac = self.line(x, line_ac);

            let start = Point::new(x, y_ac);
            let end = Point::new(x, y_ab.min(y_bc));

            self.canvas
                .draw_line(start, end)
                .expect("Failded to draw line");
        }
    }

    fn line(&self, x: i32, (p1, p2): (&Point, &Point)) -> i32 {
        let dx = p2.x - p1.x;
        if dx == 0 {
            return p1.y.max(p2.y);
        }
        let m = (p2.y - p1.y) as f32 / dx as f32;
        (m * (x - p1.x) as f32 + p1.y as f32).round() as i32
    }

    /// Doesn's include center.y
    fn elipse_top_arc(&self, x: i32, radius_x: u32, radius_y: u32, center: Position) -> i32 {
        let x = x as f32 - center.x;
        let a = radius_x as f32;
        let b = radius_y as f32;
        let y = b * (1.0 - x * x / (a * a)).max(0.0).sqrt();
        y.abs().round() as i32
    }
}
