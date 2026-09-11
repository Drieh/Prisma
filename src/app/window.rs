use sdl3::{VideoSubsystem, video::WindowFlags};

use crate::{Scene, error::PrismaError, render::Renderer, resources::ResourceManager};

pub struct AppWindow {
    pub scene: Scene,
    pub id: u32,
    renderer: Renderer,
}
impl AppWindow {
    pub fn new(
        video_subsystem: VideoSubsystem,
        builder: WindowBuilder,
        scene: Scene,
    ) -> Result<Self, PrismaError> {
        let mut binding = video_subsystem.window(&builder.title, builder.width, builder.height);

        let mut window_builder: &mut sdl3::video::WindowBuilder = binding.set_flags(builder.flags);

        if builder.is_pos_centered {
            window_builder = window_builder.position_centered();
        } else {
            window_builder = window_builder.position(builder.x, builder.y);
        }
        let window = window_builder.build().unwrap();

        Ok(Self {
            id: window.id(),
            renderer: Renderer::new(window.into_canvas()),
            scene,
        })
    }

    pub fn is_quitting(&self) -> bool {
        self.scene.is_quitting()
    }

    pub fn draw(&mut self, resources: &mut ResourceManager) {
        self.renderer.draw(&mut self.scene, resources);
    }
}
pub struct WindowBuilder {
    pub title: String,
    pub flags: WindowFlags,
    pub width: u32,
    pub height: u32,
    pub is_pos_centered: bool,
    pub x: i32,
    pub y: i32,
}
impl WindowBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            flags: WindowFlags::empty(),
            x: 0,
            y: 0,
            is_pos_centered: false,
            width: 100,
            height: 100,
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
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
        self.x = x;
        self.x = y;
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
