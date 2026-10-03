use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::renderer::{FINAL_OPTIONS, RenderOptions, render_with_options};
use crate::scene::Scene;

pub struct CompletedFrame {
    pub version: u64,
    pub generation: u64,
    pub viewport: (usize, usize),
    pub framebuffer: Framebuffer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Adaptive,
    Quality,
    Economy,
}

impl RenderMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Adaptive => "Adaptativo",
            Self::Quality => "Calidad constante",
            Self::Economy => "Económico",
        }
    }
}

pub struct RenderRequest {
    pub version: u64,
    pub generation: u64,
    pub camera: Camera,
    pub viewport: (usize, usize),
    pub mode: RenderMode,
    pub refined: bool,
    pub economy: RenderOptions,
}

#[derive(Default)]
struct State {
    pending: Option<RenderRequest>,
    completed: Option<CompletedFrame>,
    stopped: bool,
}

// Un cuadro en cálculo y una única petición pendiente, siempre la más reciente.
// La configuración viaja con cada petición, sin modificar materiales ni texturas.
#[derive(Default)]
pub struct WindowRenderer {
    state: Mutex<State>,
    wake: Condvar,
    latest_version: AtomicU64,
    generation: AtomicU64,
    stopped: AtomicBool,
}

impl WindowRenderer {
    pub fn request(&self, request: RenderRequest) {
        let mut state = self.state.lock().expect("Estado del render disponible");
        self.latest_version
            .store(request.version, Ordering::Relaxed);
        self.generation.store(request.generation, Ordering::Relaxed);
        state.pending = Some(request);
        self.wake.notify_one();
    }

    pub fn take_completed(&self) -> Option<CompletedFrame> {
        self.state
            .lock()
            .expect("Estado del render disponible")
            .completed
            .take()
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.state
            .lock()
            .expect("Estado del render disponible")
            .stopped = true;
        self.wake.notify_one();
    }

    pub fn run(&self, scene: &Scene) {
        loop {
            let request = {
                let mut state = self.state.lock().expect("Estado del render disponible");
                while state.pending.is_none() && !state.stopped {
                    state = self.wake.wait(state).expect("Estado del render disponible");
                }
                if state.stopped {
                    return;
                }
                state.pending.take().expect("Petición de render pendiente")
            };
            let preview = request.mode == RenderMode::Adaptive && !request.refined;
            let (mut width, mut height) = request.viewport;
            if preview {
                let scale = (800.0 / width as f32).min(450.0 / height as f32).min(1.0);
                width = ((width as f32 * scale).round() as usize).max(1);
                height = ((height as f32 * scale).round() as usize).max(1);
            }
            let options = match request.mode {
                RenderMode::Economy => request.economy,
                _ => RenderOptions {
                    antialias: !preview,
                    ..FINAL_OPTIONS
                },
            };
            let mut framebuffer = Framebuffer::new(width, height, Color::new(8, 10, 18));
            let cancelled = || {
                self.stopped.load(Ordering::Relaxed)
                    || self.generation.load(Ordering::Relaxed) != request.generation
                    || (request.mode == RenderMode::Adaptive
                        && request.refined
                        && self.latest_version.load(Ordering::Relaxed) != request.version)
            };
            if !render_with_options(
                &mut framebuffer,
                &request.camera,
                scene,
                options,
                &cancelled,
            ) {
                continue;
            }
            let mut state = self.state.lock().expect("Estado del render disponible");
            if state.stopped {
                return;
            }
            // Publica solo cuadros completos: nunca mezcla filas de dos cámaras.
            state.completed = Some(CompletedFrame {
                version: request.version,
                generation: request.generation,
                viewport: request.viewport,
                framebuffer,
            });
        }
    }
}
