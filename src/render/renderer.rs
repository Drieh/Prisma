use std::{collections::BTreeMap, ffi::CString, ptr::null};

use crate::{
    error::PrismaError,
    node::component::{NodeTexture, NodeVisual},
    render::math::{elipse_top_arc, line},
    resources::{FontOptions, PrismaTexture, TextureOptions, manager::ResourceManager},
    scene::{NodeID, Scene, storage::StorageHandler},
    util::{Position, Scale, Size},
};
use sdl3::{
    pixels::Color as SdlColor,
    rect::Point,
    render::Canvas,
    sys::{
        rect::SDL_FRect,
        render::{SDL_CreateTextureFromSurface, SDL_RenderTexture},
        surface::SDL_DestroySurface,
    },
    video::Window,
};
use sdl3_ttf_sys::ttf::{TTF_Font, TTF_GetStringSize, TTF_RenderText_Blended};

pub struct Renderer {
    canvas: Canvas<Window>,
}

#[expect(dead_code)]
impl Renderer {
    pub fn new(canvas: Canvas<Window>) -> Self {
        Self { canvas }
    }

    pub fn render(&mut self, scene: &mut Scene, resources: &mut ResourceManager) {
        let mut storage = scene.storage();
        let render_queue = self.build_render_queue(&mut storage);
        self.draw(render_queue, scene, resources);

        self.canvas.present();
    }

    fn build_render_queue(&self, storage: &mut StorageHandler) -> BTreeMap<usize, Vec<NodeID>> {
        let mut render_queue: BTreeMap<usize, Vec<NodeID>> = BTreeMap::new();

        for id in storage.get_nodes() {
            let tree = storage.tree.get_unchecked(id);
            let visual = storage.visual.get_unchecked(id);
            if tree.get_parent().is_none() {
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

    fn draw(
        &mut self,
        render_queue: BTreeMap<usize, Vec<NodeID>>,
        scene: &mut Scene,
        resources: &mut ResourceManager,
    ) {
        self.canvas.set_draw_color(scene.color.into_sdl_color());
        self.canvas.clear();

        for layer in render_queue.values() {
            for node in layer {
                self.draw_node(*node, scene, resources);
            }
        }
    }

    #[allow(clippy::get_first)]
    fn draw_triangle(&self, p1: Point, p2: Point, p3: Point, canvas: &mut Canvas<Window>) {
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

            canvas.draw_line(start, end).expect("Failded to draw line");
        }
    }

    fn draw_node(&mut self, id: NodeID, scene: &mut Scene, resources: &mut ResourceManager) {
        let mut node = scene
            .get_node(id)
            .expect("Internal invariant violated: render layer contains an invalid node ID");

        let Position { x: pos_x, y: pos_y } = node.get_absolute_position();
        let visual = node.get_node_visual().clone();

        let color = visual.get_bg_color();

        let Size { width, height } = visual.get_bounding_box();
        let width = width as i32;
        let height = height as i32;

        let border_radius = visual.get_border_radius();

        let Scale {
            x: scale_x,
            y: scale_y,
        } = visual.get_scale();

        let radius_x = if border_radius as f32 * scale_x > width as f32 / 2.0 {
            width / 2
        } else {
            (border_radius as f32 * scale_x).round() as i32
        };

        let radius_y = if border_radius as f32 * scale_y > height as f32 / 2.0 {
            height / 2
        } else {
            (border_radius as f32 * scale_y).round() as i32
        };

        self.canvas.set_draw_color(SdlColor {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        });

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

        let _text_result = self.render_node_text(scene, id, resources).unwrap();
    }

    fn render_node_text(
        &self,
        scene: &mut Scene,
        id: NodeID,
        resources: &mut ResourceManager,
    ) -> Result<(), PrismaError> {
        let StorageHandler { mut visual, .. } = scene.storage();
        let node_visual = visual.get_unchecked_mut(id);

        if node_visual.get_content().is_empty() {
            return Ok(());
        }

        if let Some(texture_id) = node_visual.get_texture(NodeTexture::Text) {
            let texture = resources.get_resource(texture_id, TextureOptions {})?;
            let width = texture.width;
            let height = texture.height;

            let node_position = scene.get_node(id)?.get_absolute_position();

            let rect = &SDL_FRect {
                x: node_position.x as f32,
                y: node_position.y as f32,
                w: width as f32,
                h: height as f32,
            };
            let raw = resources.get_resource(texture_id, TextureOptions {})?.raw();

            unsafe {
                let result = SDL_RenderTexture(self.canvas.raw(), raw, null(), rect);
                if !result {
                    return Err(PrismaError::RenderError(
                        "Failed to render text".to_string(),
                    ));
                }
            }
            Ok(())
        } else {
            Err(PrismaError::RenderError(format!(
                "Cannot render a node's text without an active text texture"
            )))
        }
    }

    pub fn create_text_texture(
        &self,
        visual: &NodeVisual,
        resources: &mut ResourceManager,
    ) -> Result<PrismaTexture, PrismaError> {
        if visual.get_font().is_none() {
            return Err(PrismaError::RenderError(
                "Text Renderer: cannot create a text texture without a font".to_string(),
            ));
        }
        unsafe {
            let font: *mut TTF_Font = resources
                .get_resource(
                    visual.get_font().unwrap(),
                    FontOptions {
                        size: visual.get_font_size(),
                    },
                )?
                .raw();

            let text = CString::new(visual.get_content()).unwrap();
            let mut w: i32 = 0;
            let mut h: i32 = 0;
            TTF_GetStringSize(font, text.as_ptr(), text.count_bytes(), &mut w, &mut h);

            let surface = TTF_RenderText_Blended(
                font,
                text.as_ptr(),
                text.as_bytes().len(),
                visual.get_text_color().into_sdl_color(),
            );
            if surface.is_null() {
                return Err(PrismaError::RenderError(
                    "Failed to create a SDL Surface from node's visual component".to_string(),
                ));
            }

            let texture = SDL_CreateTextureFromSurface(self.canvas.raw(), surface);

            SDL_DestroySurface(surface);

            if texture.is_null() {
                return Err(PrismaError::RenderError(
                    "Failed to create a SDL Texture from SDL_Surface".to_string(),
                ));
            }

            Ok(PrismaTexture::new(texture, w as u32, h as u32))
        }
    }
}
