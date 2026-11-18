use sdl3::{VideoSubsystem, video::WindowFlags};

use crate::{
    Scene,
    error::PrismaError,
    render::Renderer,
    resources::manager::ResourceManager,
    util::{Position, Size},
};

pub struct PrismaWindow {
    pub id: u32,
    scene: Scene,
    renderer: Renderer,
}
impl PrismaWindow {
    pub fn new(
        video_subsystem: VideoSubsystem,
        builder: WindowBuilder,
        scene: Scene,
    ) -> Result<Self, PrismaError> {
        let mut binding =
            video_subsystem.window(&builder.title, builder.size.width, builder.size.height);

        let mut window_builder: &mut sdl3::video::WindowBuilder = binding.set_flags(builder.flags);

        if builder.is_pos_centered {
            window_builder = window_builder.position_centered();
        } else {
            window_builder = window_builder.position(builder.position.x, builder.position.y);
        }
        let window = window_builder.build().unwrap();

        Ok(Self {
            id: window.id(),
            scene,
            renderer: Renderer::new(window.into_canvas()),
        })
    }

    pub fn is_quitting(&self) -> bool {
        self.scene.is_quitting()
    }

    pub fn render(&mut self, resources: &mut ResourceManager) {
        self.renderer.render(&mut self.scene, resources);
    }

    pub fn create_lifecycle_events(&mut self) {
        self.scene.create_lifecycle_events();
    }

    pub fn create_user_event(&mut self, event: sdl3::event::Event) {
        self.scene.create_user_event(&event);
    }

    pub fn dispatch_events(&mut self) {
        self.scene.dispatch_events();
    }

    pub fn process_scene(&mut self, resources: &mut ResourceManager) -> Result<(), PrismaError> {
        self.scene.process_scene(resources, &self.renderer)?;
        Ok(())
    }
}
pub struct WindowBuilder {
    pub title: String,
    pub flags: WindowFlags,
    pub size: Size,
    pub is_pos_centered: bool,
    pub position: Position,
}
impl WindowBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            flags: WindowFlags::empty(),
            size: Size {
                width: 512,
                height: 256,
            },
            is_pos_centered: false,
            position: Position { x: 100, y: 100 },
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size.height = height;
        self.size.width = width;
        self
    }

    pub fn resizable(mut self) -> Self {
        self.flags |= WindowFlags::RESIZABLE;
        self
    }

    pub fn borderless(mut self) -> Self {
        self.flags |= WindowFlags::BORDERLESS;

        self
    }

    pub fn fullscreen(mut self) -> Self {
        self.flags |= WindowFlags::FULLSCREEN;

        self
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.position.x = x;
        self.position.x = y;
        self
    }

    /**
     * This function overwrites position function.
     */
    pub fn position_centered(mut self) -> Self {
        self.is_pos_centered = true;
        self
    }
    /*
    pub fn high_pixel_density(mut self) -> Self {
        self.builder.high_pixel_density();
        self
    }
    pub fn metal_view(mut self) -> Self {
        self.builder.metal_view();
        self
    }
    pub fn hidden(mut self) -> Self {
        self.builder.hidden();
        self
    }
    pub fn opengl(mut self) -> Self {
        self.builder.opengl();
        self
    }
    pub fn input_grabbed(mut self) -> Self {
        self.builder.input_grabbed();
        self
    }
    pub fn minimized(mut self) -> Self {
        self.builder.minimized();
        self
    }
    pub fn set_flags(mut self, flags: WindowFlags) -> Self {
        self.builder.set_flags(flags);
        self
    }
    pub fn vulkan(mut self) -> Self {
        self.builder.vulkan();
        self
    }
     */
}
