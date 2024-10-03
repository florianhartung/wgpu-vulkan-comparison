//! # A wrapper for the new dumb winit architecture
//!
//! This abstration should not expose any of winit's implementation details (it's still WIP).
//! Thus winit can always be replaced by some other backend in the future.

use std::{num::NonZeroU32, sync::Arc};

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

use crate::has_window_and_display_handle::HasWindowAndDisplayHandle;

pub trait Application: Sized {
    fn new(
        window: impl HasWindowAndDisplayHandle + Send + Sync + 'static,
        initial_window_size: (u32, u32),
    ) -> Self;
    fn handle_event(&mut self, event: Event);
}

pub enum Event {
    KeyboardInput(winit::event::KeyEvent),
    ModifiersChanged(winit::event::Modifiers),
    MouseInput {
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    },
    MouseWheel(winit::event::MouseScrollDelta),
    CursorMoved((f64, f64)),
    Resize {
        size: (NonZeroU32, NonZeroU32),
        scale_factor: f64,
    },
    Render,
}

/// This makes winit fun to use again for simple single-window applications
pub fn run_window_app<T: Application>() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut wtf = Wtf::<T>::new();

    event_loop.run_app(&mut wtf).unwrap()
}

struct Wtf<T> {
    window_state: Option<(Arc<Window>, T)>,
}
impl<T> Wtf<T> {
    pub fn new() -> Self {
        Self { window_state: None }
    }
}

impl<T: Application> ApplicationHandler for Wtf<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();
        let window = Arc::new(window);

        let state = T::new(window.clone(), window.inner_size().into());

        self.window_state = Some((window, state));

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        debug_assert_eq!(
            window_id,
            self.window_state.as_ref().unwrap().0.id(),
            "there can only be one window"
        );

        let (window, state) = self.window_state.as_mut().unwrap();

        let custom_event = match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::RedrawRequested => {
                state.handle_event(Event::Render);
                window.request_redraw();
                return;
            }
            WindowEvent::Resized(physical_size) => Event::Resize {
                size: (
                    NonZeroU32::new(physical_size.width).unwrap(),
                    NonZeroU32::new(physical_size.height).unwrap(),
                ),
                scale_factor: window.scale_factor(),
            },

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => Event::Resize {
                size: (
                    NonZeroU32::new(window.inner_size().width).unwrap(),
                    NonZeroU32::new(window.inner_size().height).unwrap(),
                ),
                scale_factor,
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
                return;
            }
            WindowEvent::KeyboardInput { event, .. } => Event::KeyboardInput(event),
            WindowEvent::ModifiersChanged(modifiers) => Event::ModifiersChanged(modifiers),
            WindowEvent::CursorMoved { position, .. } => Event::CursorMoved(position.into()),
            WindowEvent::MouseWheel { delta, .. } => Event::MouseWheel(delta),
            WindowEvent::MouseInput { state, button, .. } => Event::MouseInput { state, button },
            _ => return,
        };

        state.handle_event(custom_event);
    }
}
