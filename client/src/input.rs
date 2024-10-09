use common::player::PlayerInput;
use std::collections::HashMap;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::ModifiersState;
use winit::platform::scancode::PhysicalKeyExtScancode;

#[derive(Debug, Clone, Copy)]
pub struct YawPitch{
    pub yaw: f64,
    pub pitch: f64,
}

impl YawPitch {
    //TODO: Allow mouse inverting
    pub fn update_cursor(&mut self, dx: f64, dy: f64) {
        //TODO: Don't harcode this
        let mouse_speed: f64 = 0.2;
        self.yaw -= mouse_speed * (dx as f64);
        self.pitch -= mouse_speed * (dy as f64);

        // Ensure the yaw stays within [-180; 180]
        if self.yaw < -180.0 {
            self.yaw += 360.0;
        }
        if self.yaw > 180.0 {
            self.yaw -= 360.0;
        }

        // Ensure the pitch stays within [-90; 90]
        if self.pitch < -90.0 {
            self.pitch = -90.0;
        }
        if self.pitch > 90.0 {
            self.pitch = 90.0;
        }
    }
}

impl Default for YawPitch {
    fn default() -> Self {
        Self {
            yaw: -127.0,
            pitch: -17.0,
        }
    }
}

pub struct InputState {
    keys: HashMap<u32, ElementState>,
    mouse_buttons: HashMap<MouseButton, ElementState>,
    modifiers_state: ModifiersState,
    flying: bool,
    pub enable_culling: bool,
}

impl InputState {
    pub fn new() -> InputState {
        Self {
            keys: HashMap::new(),
            mouse_buttons: HashMap::new(),
            modifiers_state: ModifiersState::default(),
            flying:true,
            enable_culling:true,
        }
    }

    pub fn process_keyboard_input(&mut self, key: KeyEvent) -> bool {
        let previous_state = self.keys.get(&key.physical_key.to_scancode().unwrap()).cloned();
        self.keys.insert(key.physical_key.to_scancode().unwrap(), key.state);
        if let &Some(ElementState::Pressed) = &previous_state {
            if key.physical_key.to_scancode().unwrap() == TOGGLE_FLIGHT {
                self.flying = !self.flying;
            }
            if key.physical_key.to_scancode().unwrap() == TOGGLE_CULLING {
                self.enable_culling = !self.enable_culling;
            }
        }
        previous_state != Some(key.state)
    }

    pub fn process_mouse_input(&mut self, button: MouseButton, state: ElementState) -> bool {
        let previous_state = self.mouse_buttons.get(&button).cloned();
        self.mouse_buttons.insert(button, state);
        previous_state != Some(state)
    }

    pub fn set_modifiers_state(&mut self, modifiers_state: ModifiersState) {
        self.modifiers_state = modifiers_state;
    }

    pub fn get_modifiers_state(&self) -> ModifiersState {
        self.modifiers_state
    }

    pub fn get_key_state(&self, key: u32) -> ElementState {
        self.keys
            .get(&key)
            .cloned()
            .unwrap_or(ElementState::Released)
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.mouse_buttons.clear();
        self.modifiers_state = ModifiersState::default();
    }

    fn is_key_pressed(&self, key: u32) -> bool {
        match self.get_key_state(key) {
            ElementState::Pressed => true,
            ElementState::Released => false,
        }
    }

    pub fn get_physics_input(&self, yaw_pitch: YawPitch, allow_movement: bool) -> PlayerInput {
        PlayerInput {
            key_move_forward: allow_movement && self.is_key_pressed(MOVE_FORWARD),
            key_move_left: allow_movement && self.is_key_pressed(MOVE_LEFT),
            key_move_backward: allow_movement && self.is_key_pressed(MOVE_BACKWARD),
            key_move_right: allow_movement && self.is_key_pressed(MOVE_RIGHT),
            key_move_up: allow_movement && self.is_key_pressed(MOVE_UP),
            key_move_down: allow_movement && self.is_key_pressed(MOVE_DOWN),
            yaw: yaw_pitch.yaw,
            pitch: yaw_pitch.pitch,
            flying: self.flying,
        }
    }
}




pub const MOVE_FORWARD: u32 = 17;
pub const MOVE_LEFT: u32 = 30;
pub const MOVE_BACKWARD: u32 = 31;
pub const MOVE_RIGHT: u32 = 32;
pub const MOVE_UP: u32 = 57;
pub const MOVE_DOWN: u32 = 42;
pub const TOGGLE_FLIGHT: u32 = 33;
pub const TOGGLE_CULLING: u32 = 46;