use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use sdl3::{Sdl, get_error as sdl_get_error};
use sdl3_ttf_sys::ttf::{TTF_Init, TTF_Quit};

use crate::{WindowBuilder, resources::ResourceManager};
use crate::{
    app::{error::PrismaError, window::AppWindow},
    scene::Scene,
};

pub struct Prisma {
    sdl_context: Sdl,
    windows: HashMap<u32, AppWindow>,
    running: bool,
    windows_close_queue: Vec<u32>,
    resources: ResourceManager,
}

impl Prisma {
    /// Returns an app builder.
    /// Is the first step using Prisma.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::InitError`] if there is any error in initialization.
    pub fn init() -> Result<Self, PrismaError> {
        let sdl_context = sdl3::init().map_err(|e| PrismaError::InitError(e.to_string()))?;
        unsafe {
            if !TTF_Init() {
                return Err(PrismaError::InitError(sdl_get_error().to_string()));
            }
        }

        Ok(Self {
            sdl_context,
            windows: HashMap::new(),
            running: false,
            windows_close_queue: Vec::new(),
            resources: ResourceManager::new(),
        })
    }

    pub fn add_window(&mut self, builder: WindowBuilder, scene: Scene) -> Result<(), PrismaError> {
        let video_subsystem = self
            .sdl_context
            .video()
            .map_err(|e| PrismaError::InitError(e.to_string()))?;
        let window = AppWindow::new(video_subsystem, builder, scene)?;
        self.windows.insert(window.id, window);
        Ok(())
    }

    fn quit(&self) {
        // SDL_Quit() is called when `sdl_context` is dropped.
        unsafe {
            TTF_Quit();
        }
    }

    /// Runs the app
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError`] if there is any error during runtime.
    pub fn run(mut self) -> Result<(), PrismaError> {
        let mut event_pump = self
            .sdl_context
            .event_pump()
            .map_err(|e| PrismaError::InitError(e.to_string()))?;
        let frame_time = Duration::from_millis(1000 / 60);
        self.running = true;
        self.resources.load_resources()?;

        while self.running {
            let frame_start = Instant::now();
            let windows_close_queue = std::mem::take(&mut self.windows_close_queue);

            for app_window in self.windows.values_mut() {
                /* events */
                app_window.scene.manage_lifecycle_events()?;
                for sdl_event in event_pump.poll_iter() {
                    if let Some(window_id) = sdl_event.get_window_id()
                        && window_id == app_window.id
                    {
                        app_window.scene.manage_sdl_events(&sdl_event)?;
                    }
                }

                /* closing queue */
                if app_window.is_quitting() {
                    self.windows_close_queue.push(app_window.id);
                }

                /* render */
                app_window.draw(&mut self.resources);
            }

            /* window close */
            for window_id in windows_close_queue {
                self.windows.remove(&window_id);
            }

            /* closing app */
            if self.windows.is_empty() {
                self.running = false;
            }

            /* frame control */
            let elapsed = frame_start.elapsed();
            if elapsed < frame_time {
                std::thread::sleep(frame_time - elapsed);
            }
        }
        Ok(())
    }
}

impl Drop for Prisma {
    fn drop(&mut self) {
        self.quit();
    }
}
