//! InputManager - handles keyboard & mouse, knows if key/mousebtn is pressed, set key timings.

use sdl2::event::Event;
use sdl2::keyboard::*;
use sdl2::mouse::*;
use std::collections::HashMap;

pub struct InputManager {
    // keep keys and buttons together, by converting their enum values to i32
    pressed: HashMap<i32, bool>,

    // TODO: I probably need pixel size, to convert to actual pizels
    // TODO: I probably need some sort of "mouse capture", to see mouse movement in window mode
    //   => just use a key for mouse capture - e.g. F12 :)
    mouse_x: i32,
    mouse_y: i32,
    // mouse movement
    mouse_rel_x: i32,
    mouse_rel_y: i32,
    pixel_size: i32,
}

impl InputManager {
    pub fn new(pixel_size: i32) -> Self {
        Self {
            pressed: HashMap::new(),
            mouse_x: 0,
            mouse_y: 0,
            mouse_rel_x: 0,
            mouse_rel_y: 0,
            pixel_size,
        }
    }

    #[inline]
    pub fn key(&self, key: Keycode) -> bool {
        let code = key2code(key);
        self.pressed.contains_key(&code)
    }

    #[inline]
    pub fn consume_key(&mut self, key: Keycode) -> bool {
        let code = key2code(key);
        self.consume_input(code)
    }

    #[inline]
    pub fn mouse_btn(&self, mb: MouseButton) -> bool {
        let code = mousebtn2code(mb);
        self.pressed.contains_key(&code)
    }

    #[inline]
    pub fn consume_mouse_btn(&mut self, mb: MouseButton) -> bool {
        let code = mousebtn2code(mb);
        self.consume_input(code)
    }

    #[inline]
    pub fn mouse_pos(&self) -> (i32, i32) {
        (self.mouse_x, self.mouse_y)
    }

    #[inline]
    pub fn consume_mouse_motion(&mut self) -> (i32, i32) {
        let ret = (self.mouse_rel_x, self.mouse_rel_y);
        self.mouse_rel_x = 0;
        self.mouse_rel_y = 0;
        ret
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(key), .. } => {
                self.set_pressed(key2code(*key));
            }
            Event::KeyUp { keycode: Some(key), .. } => {
                self.set_released(key2code(*key));
            }
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                self.mouse_x = *x / self.pixel_size;
                self.mouse_y = *y / self.pixel_size;
                self.set_pressed(mousebtn2code(*mouse_btn));
            }
            Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                self.mouse_x = *x / self.pixel_size;
                self.mouse_y = *y / self.pixel_size;
                self.set_released(mousebtn2code(*mouse_btn));
            }
            Event::MouseMotion { x, y, xrel, yrel, .. } => {
                self.mouse_x = *x / self.pixel_size;
                self.mouse_y = *y / self.pixel_size;
                self.mouse_rel_x += *xrel / self.pixel_size;
                self.mouse_rel_y += *yrel / self.pixel_size;
            }
            _ => {}
        }
    }

    #[inline]
    fn set_pressed(&mut self, keybtn: i32) {
        if !self.pressed.contains_key(&keybtn) {
            self.pressed.insert(keybtn, true);
        }
    }

    #[inline]
    fn set_released(&mut self, keybtn: i32) {
        self.pressed.remove(&keybtn);
    }

    fn consume_input(&mut self, code: i32) -> bool {
        let found = self.pressed.get_mut(&code);
        let mut pressed = false;
        if let Some(flag) = found {
            pressed = *flag;
            *flag = false;
        }
        pressed
    }
}

//------------------
//  Internal stuff

#[inline(always)]
fn key2code(key: Keycode) -> i32 {
    key as i32
}

#[inline(always)]
fn mousebtn2code(mb: MouseButton) -> i32 {
    (mb as i32) - 1000
}
