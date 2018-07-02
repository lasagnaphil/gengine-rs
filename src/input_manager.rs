use sdl2::EventPump;
use sdl2::keyboard::{KeyboardState, Keycode, Scancode, Mod};
use sdl2::mouse::{MouseState, MouseButton};
use std::collections::{HashSet, HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Key {
    Left, Right, Up, Down
}

pub enum InputEvent {
    KeyDown { key: Key, mods: Mod, repeat: bool }
}

#[derive(Clone)]
struct InputState {
    keys_down: HashSet<Key>,
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_position: (i32, i32),
}

pub struct InputManager {
    keybindings: HashMap<Scancode, Key>,
    cur_input_state: InputState,
    prev_input_state: InputState
}


impl InputManager {
    pub fn new() -> Self {
        let mut keybindings = HashMap::new();
        keybindings.insert(Scancode::Up, Key::Up);
        keybindings.insert(Scancode::Left, Key::Left);
        keybindings.insert(Scancode::Down, Key::Down);
        keybindings.insert(Scancode::Right, Key::Right);

        InputManager {
            keybindings,
            cur_input_state: InputState {
                keys_down: HashSet::new(),
                mouse_buttons_down: HashSet::new(),
                mouse_position: (0, 0)
            },
            prev_input_state: InputState {
                keys_down: HashSet::new(),
                mouse_buttons_down: HashSet::new(),
                mouse_position: (0, 0)
            },
        }
    }

    pub fn update(&mut self, event_pump: &EventPump) {
        let keys_down = event_pump.keyboard_state()
            .pressed_scancodes()
            .filter_map(|sc| { self.keybindings.get(&sc).map(|k| {*k}) })
            .collect();
        let mouse_state = event_pump.mouse_state();
        let mouse_buttons_down = mouse_state
            .mouse_buttons()
            .filter_map(|(b, p)| if p { Some(b.clone()) } else { None })
            .collect();
        let mouse_position = (mouse_state.x(), mouse_state.y());

        self.prev_input_state = self.cur_input_state.clone();
        self.cur_input_state = InputState {
            keys_down, mouse_buttons_down, mouse_position
        };
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.cur_input_state.keys_down.contains(&key)
    }

    pub fn is_key_entered(&self, key: Key) -> bool {
        !self.prev_input_state.keys_down.contains(&key) &&
            self.cur_input_state.keys_down.contains(&key)
    }

    pub fn is_key_exited(&self, key: Key) -> bool {
        self.prev_input_state.keys_down.contains(&key) &&
            !self.cur_input_state.keys_down.contains(&key)
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.cur_input_state.mouse_buttons_down.contains(&button)
    }

    pub fn is_mouse_entered(&self, button: MouseButton) -> bool {
        !self.prev_input_state.mouse_buttons_down.contains(&button) &&
            self.cur_input_state.mouse_buttons_down.contains(&button)
    }

    pub fn is_mouse_exited(&self, button: MouseButton) -> bool {
        self.prev_input_state.mouse_buttons_down.contains(&button) &&
            !self.cur_input_state.mouse_buttons_down.contains(&button)
    }

    pub fn get_mouse_pos(&self) -> (i32, i32) {
        self.cur_input_state.mouse_position
    }

    pub fn get_mouse_relative(&self) -> (i32, i32) {
        let (x, y) = self.cur_input_state.mouse_position;
        let (px, py) = self.prev_input_state.mouse_position;
        (x - px, y - py)
    }
}
