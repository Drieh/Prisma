use std::{collections::BTreeMap, ffi::CString, ptr::null};

use crate::{
    error::PrismaError,
    node::component::{NodeVisual, StyleTextureType},
    render::math::{elipse_top_arc, line},
    resources::{
        FontID, FontOptions, PrismaTexture, ResourceID, TextureID, TextureOptions,
        manager::ResourceManager,
    },
    scene::{NodeID, Scene, storage::StorageHandler},
    util::{Position, Size},
};
use sdl3::{
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

    pub fn get_viewport_size(&self) -> Size {
        let size = self.canvas.viewport().size();
        Size {
            width: size.0,
            height: size.1,
        }
    }

    pub fn render(&mut self, scene: &mut Scene, resources: &mut ResourceManager) {
        let render_queue = self.build_render_queue(scene);
        self.draw(render_queue, scene, resources);

        self.canvas.present();
    }

    fn build_render_queue(&self, scene: &mut Scene) -> BTreeMap<usize, Vec<NodeID>> {
        let mut render_queue: BTreeMap<usize, Vec<NodeID>> = BTreeMap::new();

        for id in scene.get_nodes() {
            let storage = scene.storage();
            let tree = storage.tree.get_unchecked(id);

            if tree.get_parent().is_none() {
                let parent_layer = storage.visual.get_unchecked(id).get_transform().get_layer();
                self.visit(id, scene, parent_layer, &mut render_queue);
            }
        }
        render_queue
    }

    fn visit(
        &self,
        id: NodeID,
        scene: &mut Scene,
        parent_layer: usize,
        render_queue: &mut BTreeMap<usize, Vec<NodeID>>,
    ) {
        let StorageHandler { tree, visual, .. } = scene.storage();
        let current_layer = visual.get_unchecked(id).get_transform().get_layer();

        let layer = if current_layer == 0 {
            parent_layer
        } else {
            current_layer
        };

        render_queue.entry(layer).or_default().push(id);

        for child in tree.get_unchecked(id).get_children() {
            self.visit(child, scene, layer, render_queue);
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

        let Position { x: pos_x, y: pos_y } = node.get_scene_position();
        let node_visual = node.get_visual().clone();
        let Size { width, height } = node_visual.get_bounding_box();
        let width = width as i32;
        let height = height as i32;
        let scale_x = node_visual.get_transform().get_scale_x();
        let scale_y = node_visual.get_transform().get_scale_y();
        let border_radius = node_visual.get_border().get_radius();

        let radius_x = if border_radius as f32 * scale_x > width as f32 / 2.0
            || border_radius as f32 * scale_y > height as f32 / 2.0
        {
            (width / 2).min(height / 2)
        } else {
            (border_radius as f32 * scale_x).round() as i32
        };

        let radius_y = if border_radius as f32 * scale_x > width as f32 / 2.0
            || border_radius as f32 * scale_y > height as f32 / 2.0
        {
            (width / 2).min(height / 2)
        } else {
            (border_radius as f32 * scale_y).round() as i32
        };

        let bg_color = node_visual.get_background().get_color();
        self.canvas.set_draw_color(bg_color.into_sdl_color());

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

        self.render_node_text(scene, id, resources).unwrap();
    }

    fn render_node_text(
        &self,
        scene: &mut Scene,
        id: NodeID,
        resources: &mut ResourceManager,
    ) -> Result<(), PrismaError> {
        let StorageHandler { mut visual, .. } = scene.storage();
        let node_visual = visual.get_unchecked_mut(id);
        let node_text = node_visual.get_text();

        if node_text.get_content().is_empty() {
            return Ok(());
        }

        if let Some(cached_texture) = node_visual.get_texture_cache(StyleTextureType::Text)
            && let texture_id = cached_texture.id()
            && texture_id != TextureID::none()
        {
            let texture = resources.get_resource(texture_id, TextureOptions {})?;
            let width = texture.width;
            let height = texture.height;

            let node_position = scene.get_node(id)?.get_scene_position();

            let rect = &SDL_FRect {
                x: node_position.x as f32,
                y: node_position.y as f32,
                w: width as f32,
                h: height as f32,
            };
            let raw = texture.raw();

            unsafe {
                let result = SDL_RenderTexture(self.canvas.raw(), raw, null(), rect);
                if !result {
                    return Err(PrismaError::render_error("Failed to render text"));
                }
            }
            Ok(())
        } else {
            Err(PrismaError::RenderError(format!(
                "Cannot render a node's text without an active text texture. Try to register a font first."
            )))
        }
    }

    pub fn create_text_texture(
        &self,
        visual: &NodeVisual,
        resources: &mut ResourceManager,
    ) -> Result<PrismaTexture, PrismaError> {
        let node_text = visual.get_text();

        if node_text.get_font_family() == FontID::none() {
            return Err(PrismaError::render_error(
                "Cannot create a text texture without a font.",
            ));
        }
        unsafe {
            let font: *mut TTF_Font = resources
                .get_resource(
                    node_text.get_font_family(),
                    FontOptions {
                        size: node_text.get_font_size(),
                    },
                )?
                .raw();

            let text = CString::new(node_text.get_content()).map_err(|e| {
                PrismaError::render_error(format!("Failed to create text texture: {e}"))
            })?;
            let mut w: i32 = 0;
            let mut h: i32 = 0;
            TTF_GetStringSize(font, text.as_ptr(), text.count_bytes(), &mut w, &mut h);

            let surface = TTF_RenderText_Blended(
                font,
                text.as_ptr(),
                text.as_bytes().len(),
                node_text.get_color().into_sdl_color(),
            );
            if surface.is_null() {
                return Err(PrismaError::render_error(
                    "Failed to create a SDL Surface from node's visual component",
                ));
            }

            let texture = SDL_CreateTextureFromSurface(self.canvas.raw(), surface);

            SDL_DestroySurface(surface);

            if texture.is_null() {
                return Err(PrismaError::render_error(
                    "Failed to create a SDL Texture from SDL_Surface",
                ));
            }

            Ok(PrismaTexture::new(texture, w as u32, h as u32))
        }
    }
}
