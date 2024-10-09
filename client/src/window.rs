use std::time::Instant;
use wgpu::{Device, TextureView, Surface, SurfaceConfiguration};
use anyhow::Result;
use futures::executor::block_on;
use log::{info, warn};
use texture_packer::texture::Texture;
use wgpu_types::{TextureFormat, TextureUsages};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event::WindowEvent::RedrawRequested;
use winit::event_loop;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::window::{CursorGrabMode, Window};
use crate::{
    input::InputState,
    settings::Settings
};
pub type StateFactory =
    Box<dyn FnOnce(&mut Settings, &mut Device) -> Result<(Box<dyn State>, wgpu::CommandBuffer)>>;

pub enum StateTransition {
    KeepCurrent,
    #[allow(dead_code)]
    ReplaceCurrent(StateFactory),
    CloseWindow,
}

#[derive(Debug, Clone)]
pub struct WindowData {
    pub logical_window_size: LogicalSize<f64>,
    pub physical_window_size: PhysicalSize<u32>,
    pub hidpi_factor: f64,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct WindowFlags {
    pub grab_cursor: bool,
    pub window_title: String,
}

pub trait State{
    fn update(
        &mut self,
        settings: &mut Settings,
        input_state: &InputState,
        data: &WindowData,
        flags: &mut WindowFlags,
        seconds_delta: f64,
        device: &mut Device,
    ) -> Result<StateTransition>;

    fn render<'a>(
        &mut self,
        settings: &Settings,
        buffers: WindowBuffers<'a>,
        device: &mut Device,
        data: &WindowData,
        input_state: &InputState,
    ) -> Result<(StateTransition, wgpu::CommandBuffer)>;

    fn handle_mouse_motion(&mut self, settings: Settings, delta: (f64, f64));
    fn handle_cursor_movement(&mut self, logical_position: LogicalPosition<f64>);
    fn handle_mouse_state_changes(&mut self, changes: Vec<(MouseButton, ElementState)>);
    fn handle_key_state_changes(&mut self, changes: Vec<(Option<u32>, ElementState)>);
}

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

trait ApplicationHandler<T> {
    fn handle_event(&mut self, event: &T);
}



pub fn open_window(mut settings: Settings, initial_state: StateFactory) -> () {
    info!("Opening window");
    let window_title = "MarsRobots".to_owned();
    let event_loop = EventLoop::new().unwrap();
    let window_attributes = Window::default_attributes().with_title(window_title);
    let window = event_loop.create_window(window_attributes).unwrap();
    let hidpi_factor = window.scale_factor();
    window.inner_size();
    info!("Creating the swap chain");
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(&window).unwrap();
    //Get the Device and the render Queue
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance, //TODO: Configurable ?
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
        .expect("No such adapter");
    let (mut device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: Default::default(),
    }, None))
        .expect("Unable to create device");


    let swap_chain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swap_chain_capabilities.formats[0];
    info!("Creating the multisampled texture buffer");
    let texture_view_descriptor = wgpu::TextureViewDescriptor::default();
    let mut msaa_texture_descriptor = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: surface.get_current_texture().iter().size_hint().0 as u32,
            height: surface.get_current_texture().iter().size_hint().1.unwrap() as u32,
            depth_or_array_layers: 0,
        },
        mip_level_count: 1,
        sample_count: SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::empty(),
        view_formats: &[],
    };
    let mut msaa_texture = device.create_texture(&msaa_texture_descriptor);
    let mut msaa_texture_view = msaa_texture.create_view(&texture_view_descriptor);
    info!("Creating the depth buffer");
    let mut depth_texture_descriptor = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: swapchain_format.block_dimensions().0,
            height: swapchain_format.block_dimensions().1,
            depth_or_array_layers: 0,
        },
        mip_level_count: 1,
        sample_count: SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: Default::default(),
    };
    let mut depth_texture = device.create_texture(&depth_texture_descriptor);
    let mut depth_texture_view = depth_texture.create_view(&texture_view_descriptor);

    let mut window_data = {
        let physical_window_size = window.inner_size();
        let hidpi_factor = window.scale_factor();
        let logical_window_size = physical_window_size.to_logical(hidpi_factor);
        WindowData {
            logical_window_size,
            physical_window_size,
            hidpi_factor,
            focused: false,
        }
    };
    let mut input_state = InputState::new();

    let mut window_flags = WindowFlags {
        grab_cursor: false,
        window_title: window_title.clone(),
    };

    info!("Done initializing the window. Moving on to the first state...");

    let (mut state, cmd) =
        initial_state(&mut settings, &mut device).expect("Failed to create initial window state");
    queue.submit(vec![cmd]);

    let mut previous_time = std::time::Instant::now();

    let mut window_resized = false;
    let mut mouse_state_changes = Vec::new();
    let mut key_state_changes = Vec::new();

    // Main loop
    event_loop.run_app(&mut move |event,_| {
        use winit::event::Event::*;
        match event {
            /* NORMAL EVENT HANDLING */
            WindowEvent { event, .. } => {
                use winit::event::WindowEvent::*;
                match event {
                    Resized(_) | ScaleFactorChanged { .. } => window_resized = true,
                    Moved(_) => (),
                    CloseRequested | Destroyed => (),
                    DroppedFile(_) | HoveredFile(_) | HoveredFileCancelled => (),
                    Focused(focused) => {
                        window_data.focused = focused;
                        input_state.clear();
                    }
                    KeyboardInput { event, .. } => {
                        let input = event;
                        if input_state.process_keyboard_input(input.clone()) {
                            key_state_changes.push((input.physical_key.to_scancode(), input.state));
                        }
                    }
                    CursorMoved { position, .. } => state.handle_cursor_movement(position.to_logical(hidpi_factor)),
                    CursorEntered { .. } | CursorLeft { .. } | MouseWheel { .. } => (),
                    MouseInput {
                        button,
                        state: element_state,
                        ..
                    } => {
                        if input_state.process_mouse_input(button, element_state) {
                            mouse_state_changes.push((button, element_state));
                        }
                    }
                    // weird events
                    TouchpadPressure { .. } | AxisMotion { .. } | Touch(..) | ThemeChanged(_) => (),
                    ModifiersChanged(modifiers_state) => input_state.set_modifiers_state(modifiers_state.state()),
                }
            },
            DeviceEvent { event, .. } => {
                if !window_data.focused {
                    return;
                }
                use winit::event::DeviceEvent::*;
                match event {
                    MouseMotion { delta } => state.handle_mouse_motion(settings, delta),
                    _ => (),
                }
            }
            /* MAIN LOOP TICK */
            _MainEventsCleared => {
                // If the window was resized, update the SwapChain and the window data
                if window_resized {
                    info!("The window was resized, adjusting buffers...");
                    // Update window data
                    window_data.physical_window_size = window.inner_size();
                    window_data.hidpi_factor = window.scale_factor();
                    window_data.logical_window_size = window_data.physical_window_size.to_logical(window_data.hidpi_factor);
                    // Update SwapChain
                    let config = SurfaceConfiguration {
                        usage: TextureUsages::RENDER_ATTACHMENT,
                        format: TextureFormat::R8Unorm,
                        width: window_data.physical_window_size.width,
                        height: window_data.physical_window_size.height,
                        present_mode: Default::default(),
                        desired_maximum_frame_latency: 0,
                        alpha_mode: Default::default(),
                        view_formats: Default::default(),
                    };
                    surface.configure(&device, &config);

                    // TODO: remove copy/paste
                    // Update depth buffer
                    depth_texture_descriptor.size.width = config.width;
                    depth_texture_descriptor.size.height = config.height;
                    depth_texture = device.create_texture(&depth_texture_descriptor);
                    depth_texture_view = depth_texture.create_view(&texture_view_descriptor);
                    // Udate MSAA frame buffer
                    msaa_texture_descriptor.size.width = config.width;
                    msaa_texture_descriptor.size.height = config.height;
                    msaa_texture = device.create_texture(&msaa_texture_descriptor);
                    msaa_texture_view = msaa_texture.create_view(&texture_view_descriptor);
                }
                window_resized = false;

                // Update state
                let (v1, v2) = (Vec::new(), Vec::new()); // TODO: clean up
                state.handle_mouse_state_changes(std::mem::replace(&mut mouse_state_changes, v1));
                state.handle_key_state_changes(std::mem::replace(&mut key_state_changes, v2));
                let seconds_delta = {
                    let current_time = Instant::now();
                    let delta = current_time - previous_time;
                    previous_time = current_time;
                    delta.as_secs() as f64 + delta.subsec_nanos() as f64 / 1e9
                };
                let state_transition = state
                    .update(
                        &mut settings,
                        &input_state,
                        &window_data,
                        &mut window_flags,
                        seconds_delta,
                        &mut device,
                    )
                    .expect("Failed to `update` the current window state"); // TODO: remove this

                // Update window flags
                window.set_title(&window_flags.window_title);
                if window_flags.grab_cursor && window_data.focused {
                    window.set_cursor_visible(false);
                    let PhysicalSize { width, height } = window_data.physical_window_size;
                    let center_pos = PhysicalPosition { x: width / 2, y: height / 2 };
                    match window.set_cursor_grab(CursorGrabMode::Locked) {
                        Err(err) => warn!("Failed to grab cursor ({:?})", err),
                        _ => (),
                    }
                    match window.set_cursor_position(center_pos) {
                        Err(err) => warn!("Failed to center cursor ({:?})", err),
                        _ => (),
                    }
                } else {
                    window.set_cursor_visible(true);
                    match window.set_cursor_grab(CursorGrabMode::None) {
                        Err(err) => warn!("Failed to ungrab cursor ({:?})", err),
                        _ => (),
                    }
                }

                // Transition if necessary
                match state_transition {
                    StateTransition::KeepCurrent => (),
                    StateTransition::ReplaceCurrent(new_state) => {
                        info!("Transitioning to a new window state...");
                        let (new_state, cmd) = new_state(&mut settings, &mut device)
                            .expect("Failed to create next window state");
                        state = new_state;
                        queue.submit(vec![cmd]);
                        return;
                    }
                    StateTransition::CloseWindow => {

                    }
                }

                // Render frame
                let swap_chain_output = surface.get_current_texture().expect("Failed to unwrap swap chain output.");
                let (state_transition, commands) = state
                    .render(
                        &settings,
                        WindowBuffers {
                            texture_buffer: &swap_chain_output.texture.create_view(&Default::default()),
                            multisampled_texture_buffer: &msaa_texture_view,
                            depth_buffer: &depth_texture_view,
                        },
                        &mut device,
                        &window_data,
                        &input_state,
                    )
                    .expect("Failed to `render` the current window state");
                queue.submit(vec![commands]);
                match state_transition {
                    StateTransition::KeepCurrent => (),
                    StateTransition::ReplaceCurrent(new_state) => {
                        let (new_state, cmd) = new_state(&mut settings, &mut device)
                            .expect("Failed to create next window state");
                        state = new_state;
                        queue.submit(vec![cmd]);
                    }
                    StateTransition::CloseWindow => {
                        ();
                    }
                }
            }
            // TODO: handle this
            LoopDestroyed => {
                // TODO: cleanup relevant stuff
            }
            _ => (),
        }
    }).expect("TODO: panic message")
}

pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.2,
    g: 0.2,
    b: 0.2,
    a: 1.0,
};

pub const CLEAR_DEPTH: f32 = 1.0;
pub  const SAMPLE_COUNT: u32 = 4;


#[derive(Debug, Clone, Copy)]
pub struct WindowBuffers<'a> {
    pub texture_buffer: &'a TextureView,
    pub multisampled_texture_buffer: &'a TextureView,
    pub depth_buffer: &'a TextureView,
}