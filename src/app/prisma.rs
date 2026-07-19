use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use sdl3::{VideoSubsystem, render::Canvas, video::WindowFlags};

use crate::render::Renderer;
use crate::{app::PrismaError, scene::Scene};

pub struct AppWindow {
    scene: Scene,
    renderer: Renderer,
    id: u32,
}
impl AppWindow {
    pub fn new(
        video_subsystem: VideoSubsystem,
        builder: WindowBuilder,
        scene: Scene,
    ) -> Result<Self, PrismaError> {
        let mut binding =
            video_subsystem.window(&builder.title.into_string(), builder.width, builder.height);

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

    pub fn draw(&mut self) {
        self.renderer.draw(&self.scene);
    }
}
pub struct WindowBuilder {
    pub title: Box<str>,
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
            title: title.into(),
            flags: WindowFlags::empty(),
            x: 0,
            y: 0,
            is_pos_centered: false,
            width: 100,
            height: 100,
        }
    }
    //pub fn build(self) -> sdl3::video::Window {}
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
    pub fn position_centered(mut self) -> Self {
        self.builder.position_centered();
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

pub struct Prisma {
    event_pump: sdl3::EventPump,
    windows: HashMap<u32, AppWindow>,
    running: bool,
    windows_close_queue: Vec<u32>,
}
impl Prisma {
    /**
     * Returns a
     */
    pub fn builder() -> Result<AppBuilder, PrismaError> {
        let sdl_context = sdl3::init().map_err(|e| PrismaError::InitError(e.to_string()))?;

        let video_subsystem = sdl_context
            .video()
            .map_err(|e| PrismaError::InitError(e.to_string()))?;

        Ok(AppBuilder::new(video_subsystem, sdl_context))
    }

    pub fn run(mut self) -> Result<(), PrismaError> {
        self.running = true;
        let frame_time = Duration::from_millis(1000 / 60);

        while self.running {
            let frame_start = Instant::now();
            let windows_close_queue = std::mem::take(&mut self.windows_close_queue);

            // life cycle events
            for app_window in self.windows.values_mut() {
                app_window
                    .scene
                    .manage_lifecycle()
                    .expect("Error in lifecycle");
            }
            // sdl user events
            for sdl_event in self.event_pump.poll_iter() {
                for app_window in self.windows.values_mut() {
                    if let Some(window_id) = sdl_event.get_window_id() {
                        if window_id == app_window.id {
                            app_window
                                .scene
                                .manage_sdl_event(&sdl_event)
                                .expect("Error in user events");
                        }
                    }
                }
            }
            // window close management
            for window_id in self.windows.keys().clone() {
                if self.windows.get(window_id).unwrap().is_quitting() {
                    self.windows_close_queue.push(*window_id);
                }
            }
            for window_id in windows_close_queue {
                self.windows.remove(&window_id);
            }

            // render
            for app_window in self.windows.values_mut() {
                app_window.draw();
            }

            // closing app
            if self.windows.len() == 0 {
                self.running = false;
            }

            let elapsed = frame_start.elapsed();
            if elapsed < frame_time {
                std::thread::sleep(frame_time - elapsed);
            }
        }
        Ok(())
    }
}
pub struct AppBuilder {
    video_subsystem: VideoSubsystem,
    sdl_context: sdl3::Sdl,
    windows: Vec<AppWindow>,
}
impl AppBuilder {
    pub fn new(video_subsystem: VideoSubsystem, sdl_context: sdl3::Sdl) -> Self {
        Self {
            video_subsystem,
            sdl_context: sdl_context,
            windows: Vec::new(),
        }
    }

    pub fn window(
        mut self,
        window_builder: WindowBuilder,
        scene: Scene,
    ) -> Result<Self, PrismaError> {
        let window = AppWindow::new(self.video_subsystem.clone(), window_builder, scene)?;

        self.windows.push(window);

        Ok(self)
    }

    pub fn build(self) -> Result<Prisma, PrismaError> {
        let windows = self
            .windows
            .into_iter()
            .map(|window| (window.id, window))
            .collect();

        Ok(Prisma {
            event_pump: self
                .sdl_context
                .event_pump()
                .map_err(|e| PrismaError::InitError(e.to_string()))?,
            windows,
            running: false,
            windows_close_queue: Vec::new(),
        })
    }
}
