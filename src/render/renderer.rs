use std::collections::BTreeMap;

use crate::{
    render::math::{elipse_top_arc, line},
    resources::ResourceManager,
    scene::{NodeID, Scene, storage::StorageHandler},
    util::{Position, Scale, Size},
};
use sdl3::{pixels::Color as SdlColor, rect::Point, render::Canvas, video::Window};

pub struct Renderer {
    canvas: Canvas<Window>,
}
#[expect(dead_code)]
impl Renderer {
    pub fn new(canvas: Canvas<Window>) -> Self {
        let mut _self = Self { canvas };
        _self.canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        _self
    }

    pub fn draw(&mut self, scene: &mut Scene, _resources: &mut ResourceManager) {
        let mut storage = scene.storage();
        let render_queue = self.build_render_layers(&mut storage);

        self.render(render_queue, scene);
        self.canvas.present();
    }

    fn build_render_layers(
        &mut self,
        storage: &mut StorageHandler,
    ) -> BTreeMap<usize, Vec<NodeID>> {
        let mut render_queue: BTreeMap<usize, Vec<NodeID>> = BTreeMap::new();

        for id in storage.get_nodes() {
            let node_tree = storage.tree.get_unchecked(id);
            let visual = storage.visual.get_unchecked(id);
            if node_tree.get_parent().is_none() {
                self.visit(id, storage, visual.get_layer(), &mut render_queue);
            }
        }
        render_queue
    }
    fn visit(
        &self,
        id: NodeID,
        storage: &mut StorageHandler,
        parent_layer: Option<usize>,
        render_queue: &mut BTreeMap<usize, Vec<NodeID>>,
    ) {
        let StorageHandler { tree, visual, .. } = storage;
        let layer = visual
            .get_unchecked(id)
            .get_layer()
            .or(parent_layer)
            .unwrap_or(0);

        render_queue.entry(layer).or_default().push(id);

        for child in tree.get_unchecked(id).get_children() {
            self.visit(child, storage, Some(layer), render_queue);
        }
    }
    fn render(&mut self, render_queue: BTreeMap<usize, Vec<NodeID>>, scene: &mut Scene) {
        self.canvas.set_draw_color(scene.color.into_sdl_color());
        self.canvas.clear();

        for layer in render_queue.values() {
            for node_id in layer {
                self.render_node(*node_id, scene);
            }
        }
    }

    #[allow(clippy::get_first)]
    fn draw_triangle(&mut self, p1: Point, p2: Point, p3: Point) {
        let mut points: Vec<Point> = vec![p1, p2, p3];
        points.sort_by_key(|p| p.x);

        let point_a = points.get(0).unwrap();
        let point_b = points.get(1).unwrap();
        let point_c = points.get(2).unwrap();

        let line_ab = (point_a, point_b);
        let line_bc = (point_b, point_c);
        let line_ac = (point_a, point_c);

        for x in point_a.x..=point_c.x {
            let y_ab = line(x, line_ab);
            let y_bc = line(x, line_bc);
            let y_ac = line(x, line_ac);

            let start = Point::new(x, y_ac);
            let end = Point::new(x, y_ab.min(y_bc));

            self.canvas
                .draw_line(start, end)
                .expect("Failded to draw line");
        }
    }

    fn render_node(&mut self, id: NodeID, scene: &mut Scene) {
        let mut node = scene
            .get_node(id)
            .expect("Internal invariant violated: render layer contains an invalid node ID");

        let visual = *node.get_visual();

        let color = node.get_visual().get_color();
        self.canvas.set_draw_color(SdlColor {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        });

        let Size { width, height } = node.get_bounding_box_size();
        let width = width as i32;
        let height = height as i32;
        let Position { x: pos_x, y: pos_y } = node.get_absolute_position();
        let border_radius = visual.get_border_radius();
        let Scale {
            x: scale_x,
            y: scale_y,
        } = visual.get_scale();

        let radius_x = if border_radius as f32 * scale_x > width as f32 / 2.0 {
            width / 2
        } else if border_radius as f32 * scale_y > height as f32 / 2.0 {
            height / 2
        } else {
            (border_radius as f32 * scale_x).round() as i32
        };

        let radius_y = if border_radius as f32 * scale_x > width as f32 / 2.0 {
            width / 2
        } else if border_radius as f32 * scale_y > height as f32 / 2.0 {
            height / 2
        } else {
            (border_radius as f32 * scale_y).round() as i32
        };

        for x in (pos_x)..=(pos_x + width as i32) {
            let mut start = Point::new(x, 0);
            let mut end = Point::new(x, 0);

            // Esquinas izquierdas
            if x <= pos_x + radius_x {
                let top_center = Position {
                    x: pos_x + radius_x,
                    y: pos_y + radius_y,
                };
                let bottom_center = Position {
                    x: pos_x + radius_x,
                    y: pos_y - radius_y + height,
                };
                start.y = -elipse_top_arc(x, radius_x, radius_y, top_center) + pos_y + radius_y;

                end.y = elipse_top_arc(x, radius_x, radius_y, bottom_center) + pos_y - radius_y
                    + height;
            }
            // Esquinas derechas
            else if x as i32 >= pos_x + width as i32 - radius_x as i32 {
                let center = Position {
                    x: pos_x + width as i32 - radius_x as i32,
                    y: pos_y + radius_y as i32,
                };

                start.y =
                    -elipse_top_arc(x, radius_x, radius_y, center) + pos_y as i32 + radius_y as i32;

                end.y =
                    elipse_top_arc(x, radius_x, radius_y, center) + pos_y as i32 + height as i32
                        - radius_y as i32;
            }
            // Parte recta
            else {
                start.y = pos_y;
                end.y = pos_y + height;
            }

            self.canvas
                .draw_line(start, end)
                .expect("Failed to draw line");
        }
        self.render_text(id, scene);
    }

    #[expect(unused)]
    fn render_text(&mut self, id: NodeID, scene: &mut Scene) {}
}
