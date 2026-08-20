use chrono::{Local, Timelike};
use gamepads::*;
use macroquad::{
    miniquad::window::{dpi_scale, screen_size, set_window_position},
    prelude::*,
};
use std::process;
use system_shutdown::shutdown;
use volumecontrol::AudioDevice;

/// InputManager manages Joystick Analog Sticks, DPAD, Keyboard,
/// and smooth key-repeat debouncing.
struct InputManager {
    gp: Gamepads,
    deadzone: f32,
    repeat_delay: f64,    // Time before repeat kicks in (seconds)
    repeat_interval: f64, // Time between repeated steps (seconds)

    // Direction repeat timers
    up_timer: DirectionTimer,
    down_timer: DirectionTimer,
    left_timer: DirectionTimer,
    right_timer: DirectionTimer,
}

#[derive(Default, Clone, Copy)]
struct DirectionTimer {
    is_held: bool,
    press_time: f64,
    last_repeat: f64,
}

impl InputManager {
    fn new() -> Self {
        Self {
            gp: Gamepads::new(),
            deadzone: 0.45,        // Stick deadzone (prevents drift)
            repeat_delay: 0.28,    // 280ms initial hold delay
            repeat_interval: 0.10, // 100ms repeat rate (10 steps/sec)
            up_timer: DirectionTimer::default(),
            down_timer: DirectionTimer::default(),
            left_timer: DirectionTimer::default(),
            right_timer: DirectionTimer::default(),
        }
    }

    /// MUST be called strictly ONCE at the start of each frame
    fn poll_frame(&mut self) {
        self.gp.poll();
    }

    /// Directional trigger helper with initial delay and repeat intervals
    fn process_direction(
        timer: &mut DirectionTimer,
        is_currently_down: bool,
        current_time: f64,
        delay: f64,
        interval: f64,
    ) -> bool {
        if is_currently_down {
            if !timer.is_held {
                // First press: trigger immediately
                timer.is_held = true;
                timer.press_time = current_time;
                timer.last_repeat = current_time;
                true
            } else {
                // Held down: check repeat timers
                let hold_duration = current_time - timer.press_time;
                if hold_duration >= delay && (current_time - timer.last_repeat) >= interval {
                    timer.last_repeat = current_time;
                    true
                } else {
                    false
                }
            }
        } else {
            timer.is_held = false;
            false
        }
    }

    pub fn is_up_triggered(&mut self, current_time: f64) -> bool {
        let kb = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
        let dpad = self
            .gp
            .all()
            .any(|g| g.is_currently_pressed(Button::DPadUp));
        // Analog Left Stick Y-axis
        let stick = self.gp.all().any(|g| {
            let (_x, y) = g.left_stick();
            y > self.deadzone
        });

        let is_down = kb || dpad || stick;
        Self::process_direction(
            &mut self.up_timer,
            is_down,
            current_time,
            self.repeat_delay,
            self.repeat_interval,
        )
    }

    pub fn is_down_triggered(&mut self, current_time: f64) -> bool {
        let kb = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
        let dpad = self
            .gp
            .all()
            .any(|g| g.is_currently_pressed(Button::DPadDown));
        let stick = self.gp.all().any(|g| {
            let (_x, y) = g.left_stick();
            y < -self.deadzone
        });

        let is_down = kb || dpad || stick;
        Self::process_direction(
            &mut self.down_timer,
            is_down,
            current_time,
            self.repeat_delay,
            self.repeat_interval,
        )
    }

    pub fn is_left_triggered(&mut self, current_time: f64) -> bool {
        let kb = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        let dpad = self
            .gp
            .all()
            .any(|g| g.is_currently_pressed(Button::DPadLeft));
        let stick = self.gp.all().any(|g| {
            let (x, _y) = g.left_stick();
            x < -self.deadzone
        });

        let is_down = kb || dpad || stick;
        Self::process_direction(
            &mut self.left_timer,
            is_down,
            current_time,
            self.repeat_delay,
            0.06, // Faster volume ramp (60ms repeat)
        )
    }

    pub fn is_right_triggered(&mut self, current_time: f64) -> bool {
        let kb = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        let dpad = self
            .gp
            .all()
            .any(|g| g.is_currently_pressed(Button::DPadRight));
        let stick = self.gp.all().any(|g| {
            let (x, _y) = g.left_stick();
            x > self.deadzone
        });

        let is_down = kb || dpad || stick;
        Self::process_direction(
            &mut self.right_timer,
            is_down,
            current_time,
            self.repeat_delay,
            0.06, // Faster volume ramp (60ms repeat)
        )
    }

    pub fn is_enter_pressed(&self) -> bool {
        is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || self.gp.all().any(|g| {
                g.is_just_pressed(Button::ActionDown) // A / Cross button
            })
    }
}

#[macroquad::main("CoralliumConfig")]
async fn main() {
    set_pc_assets_folder("/home/main/corallium_config");
    let mut auddevice = AudioDevice::from_default().ok();
    let mut selected_option: i32 = 0;
    let mut volume: f64 = if let Some(ref mut dev) = auddevice {
        dev.get_vol().unwrap_or(50) as f64 / 100.0
    } else {
        0.5
    };
    let battery_level: f64 = 1.00;

    // Load textures safely
    let bg_texture = load_texture("assets/menu.png")
        .await
        .unwrap_or_else(|_| Texture2D::empty());
    let textures = vec![
        load_texture("assets/quit_select.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/vol_select.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/shutdown_select.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/reset_select.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/numbers.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
    ];
    for texture in &textures {
        texture.set_filter(FilterMode::Nearest);
    }

    let mut input = InputManager::new();

    let aspect_ratio = if bg_texture.height() > 0.0 {
        bg_texture.width() / bg_texture.height()
    } else {
        16.0 / 9.0
    };
    let height = screen_size().1 * dpi_scale();
    let target_width = height * aspect_ratio;
    let scale_factor = if bg_texture.height() > 0.0 {
        height / bg_texture.height()
    } else {
        1.0
    };

    request_new_screen_size(target_width, height);
    set_window_position(10000, 0);
    bg_texture.set_filter(FilterMode::Nearest);

    let volume_textures = [
        load_texture("assets/volume_0.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_10.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_20.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_30.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_40.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_50.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_60.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_70.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_80.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_90.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/volume_100.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
    ];
    for texture in &volume_textures {
        texture.set_filter(FilterMode::Nearest);
    }

    let battery_textures = [
        load_texture("assets/battery_0.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/battery_16.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/battery_33.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/battery_50.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/battery_66.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/battery_83.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
        load_texture("assets/battery_100.png")
            .await
            .unwrap_or_else(|_| Texture2D::empty()),
    ];
    for texture in &battery_textures {
        texture.set_filter(FilterMode::Nearest);
    }

    loop {
        let current_time = get_time();

        // 1. Single poll per frame
        input.poll_frame();

        if let Some(ref mut dev) = auddevice {
            let _ = dev.set_vol((volume * 100.0).round() as u8);
        }

        // 2. Update real-time clock every frame
        let now = Local::now();
        let hr = now.hour();
        let min = now.minute();

        clear_background(BLACK);
        draw_texture_ex(
            &bg_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(target_width, height)),
                ..Default::default()
            },
        );

        // 3. Navigation with Joystick + DPAD + Key repeat
        if input.is_down_triggered(current_time) {
            selected_option = (selected_option + 1) % 4;
        }
        if input.is_up_triggered(current_time) {
            // Fix: Adding 3 and modulo 4 avoids negative remainder bug
            selected_option = (selected_option + 3) % 4;
        }

        // 4. Render Active Selection
        match selected_option {
            0 => {
                draw_texture_ex(
                    &textures[0],
                    (111.0 / bg_texture.width()) * target_width,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(
                            textures[0].width() * scale_factor,
                            textures[0].height() * scale_factor,
                        )),
                        ..Default::default()
                    },
                );
            }
            1 => {
                draw_texture_ex(
                    &textures[1],
                    (10.0 / bg_texture.width()) * target_width,
                    (55.0 / bg_texture.height()) * height,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(
                            textures[1].width() * scale_factor,
                            textures[1].height() * scale_factor,
                        )),
                        ..Default::default()
                    },
                );
            }
            2 => {
                draw_texture_ex(
                    &textures[2],
                    (11.0 / bg_texture.width()) * target_width,
                    (149.0 / bg_texture.height()) * height,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(
                            textures[2].width() * scale_factor,
                            textures[2].height() * scale_factor,
                        )),
                        ..Default::default()
                    },
                );
            }
            3 => {
                draw_texture_ex(
                    &textures[3],
                    (11.0 / bg_texture.width()) * target_width,
                    (189.0 / bg_texture.height()) * height,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(
                            textures[3].width() * scale_factor,
                            textures[3].height() * scale_factor,
                        )),
                        ..Default::default()
                    },
                );
            }
            _ => {}
        }

        // 5. Actions & Volume Controls
        match selected_option {
            0 => {
                if input.is_enter_pressed() {
                    process::exit(0);
                }
            }
            1 => {
                if input.is_left_triggered(current_time) {
                    volume = (volume - 0.05).max(0.0);
                }
                if input.is_right_triggered(current_time) {
                    volume = (volume + 0.05).min(1.0);
                }
            }
            2 => {
                if input.is_enter_pressed() {
                    if let Err(e) = shutdown() {
                        eprintln!("Couldn't shutdown: {:?}", e);
                    }
                    return;
                }
            }
            3 => {
                if input.is_enter_pressed() {
                    process::exit(4);
                }
            }
            _ => {}
        }

        // 6. Draw Gauges & Clock Digits
        let vol_index = (volume * 10.0).round().clamp(0.0, 10.0) as usize;
        let battery_index = (battery_level * 6.0).round().clamp(0.0, 6.0) as usize;

        draw_texture_ex(
            &volume_textures[vol_index],
            (53.0 / bg_texture.width()) * target_width,
            (64.0 / bg_texture.height()) * height,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    volume_textures[vol_index].width() * scale_factor,
                    volume_textures[vol_index].height() * scale_factor,
                )),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &battery_textures[battery_index],
            (17.0 / bg_texture.width()) * target_width,
            (106.0 / bg_texture.height()) * height,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    battery_textures[battery_index].width() * scale_factor,
                    battery_textures[battery_index].height() * scale_factor,
                )),
                ..Default::default()
            },
        );

        let batt_pct = (battery_level * 100.0).round() as u32;
        if batt_pct >= 100 {
            draw_digit(
                1,
                (50.0 / bg_texture.width()) * target_width,
                (113.0 / bg_texture.height()) * height,
                scale_factor,
                &textures[4],
            );
        }
        draw_digit(
            (batt_pct % 100) / 10,
            (67.0 / bg_texture.width()) * target_width,
            (113.0 / bg_texture.height()) * height,
            scale_factor,
            &textures[4],
        );
        draw_digit(
            batt_pct % 10,
            (84.0 / bg_texture.width()) * target_width,
            (113.0 / bg_texture.height()) * height,
            scale_factor,
            &textures[4],
        );

        draw_number(
            hr,
            (7.0 / bg_texture.width()) * target_width,
            (2.0 / bg_texture.height()) * height,
            scale_factor,
            &textures[4],
        );
        draw_number(
            min,
            (45.0 / bg_texture.width()) * target_width,
            (2.0 / bg_texture.height()) * height,
            scale_factor,
            &textures[4],
        );

        next_frame().await
    }
}

fn draw_number(num: u32, mut x: f32, y: f32, scale_factor: f32, tex: &Texture2D) {
    let mut num_str = num.to_string();
    let digit_width = 14.0 * scale_factor;
    let spacing = 3.0 * scale_factor;
    if num_str.len() == 1 {
        num_str = format!("0{}", num_str);
    }
    for ch in num_str.chars() {
        if let Some(digit) = ch.to_digit(10) {
            draw_digit(digit, x, y, scale_factor, tex);
            x += digit_width + spacing;
        }
    }
}

fn draw_digit(num: u32, x: f32, y: f32, scale_factor: f32, tex: &Texture2D) {
    let src_x = 14 * (num % 5);
    let src_y = (18.0 * (num as f32 / 5.0).floor()) as i32;
    let src_rect = Rect::new(src_x as f32, src_y as f32, 14.0, 18.0);
    draw_texture_ex(
        tex,
        x,
        y,
        WHITE,
        DrawTextureParams {
            source: Some(src_rect),
            dest_size: Some(vec2(14.0 * scale_factor, 19.0 * scale_factor)),
            ..Default::default()
        },
    );
}
