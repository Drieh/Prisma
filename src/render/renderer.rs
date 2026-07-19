use crate::{common::Position, scene::NodeID, scene::Scene};
use sdl3::{render::Canvas, video::Window};

pub struct Renderer {
    canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new(canvas: Canvas<Window>) -> Self {
        Self { canvas }
    }

    pub fn draw(&mut self, scene: &Scene) {
        let mut layers: Vec<usize> = Vec::new();
        for node in scene.nodes().values() {
            let node_layer = node.get_transform().layer.unwrap_or(0);
            if !layers.contains(&node_layer) {
                layers.push(node_layer);
            }
        }
        let render_queue: Vec<Vec<NodeID>> = self.build_render_layers(layers, scene);

        self.render(render_queue, scene);
        self.canvas.present();
        return;
    }

    pub fn build_render_layers(&mut self, layers: Vec<usize>, scene: &Scene) -> Vec<Vec<NodeID>> {
        // optimizar sistema de layers. guardar solo layers que existen en vez de recorrer hasta la maxima
        let mut render_queue: Vec<Vec<NodeID>> = Vec::new();
        for layer in layers {
            let mut layer_queue: Vec<NodeID> = Vec::new();
            for node in scene.nodes().values() {
                if node.get_transform().layer.unwrap_or(0) != layer {
                    continue;
                }
                layer_queue.push(node.id());
            }
            let sorted_queue = self.sort_render_layer(&mut layer_queue, scene);
            render_queue.push(sorted_queue);
        }
        render_queue
    }

    fn sort_render_layer(&mut self, layer: &[NodeID], scene: &Scene) -> Vec<NodeID> {
        let mut sorted = Vec::new();

        for &current_node_id in layer {
            let is_layer_root = if let Some(parent_node) = scene
                .get_node(current_node_id)
                .expect("Internal invariant violated: render layer contains an invalid node ID")
                .get_parent()
            {
                scene
                    .get_node(parent_node)
                    .expect("Internal invariant violated: render layer contains an invalid node ID")
                    .get_transform()
                    .layer
                    .unwrap_or(0)
                    != scene
                        .get_node(current_node_id)
                        .expect(
                            "Internal invariant violated: render layer contains an invalid node ID",
                        )
                        .get_transform()
                        .layer
                        .unwrap_or(0)
            } else {
                true
            };
            if is_layer_root {
                self.push_render_node_family(current_node_id, layer, &mut sorted, scene);
            }
        }
        return sorted;
    }

    fn push_render_node_family(
        &mut self,
        node_id: NodeID,
        layer: &[NodeID],
        output: &mut Vec<NodeID>,
        scene: &Scene,
    ) {
        output.push(node_id);

        let node = scene
            .get_node(node_id)
            .expect("Internal invariant violated: render layer contains an invalid node ID");

        for child_id in &node.get_children().clone() {
            if layer.contains(child_id) {
                self.push_render_node_family(*child_id, layer, output, scene);
            }
        }
    }

    fn render(&mut self, render_queue: Vec<Vec<NodeID>>, scene: &Scene) {
        self.canvas.set_draw_color(scene.color);
        self.canvas.clear();

        for layer in render_queue {
            for node_id in layer {
                let world_position = self.node_world_position(node_id, scene);
                scene
                    .get_node(node_id)
                    .expect("Internal invariant violated: render layer contains an invalid node ID")
                    .draw(&mut self.canvas, world_position);
            }
        }
    }

    pub fn node_world_position(&self, id: NodeID, scene: &Scene) -> Position {
        let node = scene.nodes().get(&id).expect("Invalid Node ID.");

        if let Some(parent_id) = node.get_parent() {
            self.node_world_position(parent_id, scene) + node.get_transform().position
        } else {
            node.get_transform().position
        }
    }
}
