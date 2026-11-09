use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};

use sdl3::{Sdl, get_error as sdl_get_error};
use sdl3_ttf_sys::ttf::{TTF_Init, TTF_Quit};

use crate::{
    WindowBuilder,
    resources::{Resource, manager::ResourceManager},
};
use crate::{
    app::{error::PrismaError, window::PrismaWindow},
    scene::Scene,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub(crate) struct CloseRequest {
    pub duration: std::time::Duration,
    pub requested_at: std::time::Instant,
}

pub struct Prisma {
    resources: ResourceManager,
    windows: HashMap<u32, PrismaWindow>,
    windows_close_queue: Vec<u32>,
    running: bool,
    sdl_context: Sdl,
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
        let window = PrismaWindow::new(video_subsystem, builder, scene)?;
        self.windows.insert(window.id, window);
        Ok(())
    }

    pub fn register<T>(&mut self, path: impl AsRef<Path>) -> Result<T::ResourceID, PrismaError>
    where
        T: Resource,
    {
        let path = path.as_ref();

        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| {
                    PrismaError::ResourceNotFound(
                        "Prisma: cannot create a registry from a non existing resource".to_string(),
                    )
                })?
                .join(path)
        };
        self.resources.register_external::<T>(&absolute_path)
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

        for window in self.windows.values_mut() {
            window.process_scene(&mut self.resources);
        }

        while self.running {
            let frame_start = Instant::now();
            let windows_close_queue = std::mem::take(&mut self.windows_close_queue);

            for window in self.windows.values_mut() {
                /* events */
                window.create_lifecycle_events();

                for sdl_event in event_pump.poll_iter() {
                    if let Some(window_id) = sdl_event.get_window_id()
                        && window_id == window.id
                    {
                        window.create_user_event(sdl_event);
                    }
                }
                window.dispatch_events();
                window.process_scene(&mut self.resources)?;

                /* closing queue */

                // conexion para eventos globales para comunicacion entre ventanas
                if window.is_quitting() {
                    self.windows_close_queue.push(window.id);
                }

                /* render */
                window.render(&mut self.resources);
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
        // SDL_Quit() is called when `sdl_context` is dropped.
        self.resources.clear();
        unsafe {
            TTF_Quit();
        }
    }
}
