use chrono::{Local, Timelike};
use gamepads::*;
use macroquad::{
    miniquad::window::{dpi_scale, screen_size, set_window_position},
    prelude::*,
};
use std::process;
use system_shutdown::shutdown;
use volumecontrol::AudioDevice;
#[macroquad::main("CoralliumConfig")]
async fn main() {
    let auddevice = AudioDevice::from_default().expect("couldn't find audio device");
    let mut selected_option = 0;
    let mut volume = auddevice.get_vol().expect("couldnt get vol") as f64 / 100.0;
    let battery_level = 1.00;
    let now = Local::now();
    let bg_texture = load_texture("assets/menu.png").await.unwrap();
    let textures = vec![
        load_texture("assets/quit_select.png").await.unwrap(),
        load_texture("assets/vol_select.png").await.unwrap(),
        load_texture("assets/shutdown_select.png").await.unwrap(),
        load_texture("assets/reset_select.png").await.unwrap(),
        load_texture("assets/numbers.png").await.unwrap(),
    ];
    for texture in &textures {
        texture.set_filter(FilterMode::Nearest);
    }
    let aspect_ratio = bg_texture.width() / bg_texture.height();
    let height = screen_size().1 * dpi_scale();
    let target_width = height * aspect_ratio;
    let scale_factor = height / bg_texture.height();
    request_new_screen_size(target_width, height);
    set_window_position(10000, 0);
    bg_texture.set_filter(FilterMode::Nearest);
    let volume_textures = [
        load_texture("assets/volume_0.png").await.unwrap(),
        load_texture("assets/volume_10.png").await.unwrap(),
        load_texture("assets/volume_20.png").await.unwrap(),
        load_texture("assets/volume_30.png").await.unwrap(),
        load_texture("assets/volume_40.png").await.unwrap(),
        load_texture("assets/volume_50.png").await.unwrap(),
        load_texture("assets/volume_60.png").await.unwrap(),
        load_texture("assets/volume_70.png").await.unwrap(),
        load_texture("assets/volume_80.png").await.unwrap(),
        load_texture("assets/volume_90.png").await.unwrap(),
        load_texture("assets/volume_100.png").await.unwrap(),
    ];
    for texture in &volume_textures {
        texture.set_filter(FilterMode::Nearest);
    }
    let battery_textures = [
        load_texture("assets/battery_0.png").await.unwrap(),
        load_texture("assets/battery_16.png").await.unwrap(),
        load_texture("assets/battery_33.png").await.unwrap(),
        load_texture("assets/battery_50.png").await.unwrap(),
        load_texture("assets/battery_66.png").await.unwrap(),
        load_texture("assets/battery_83.png").await.unwrap(),
        load_texture("assets/battery_100.png").await.unwrap(),
    ];
    for texture in &battery_textures {
        texture.set_filter(FilterMode::Nearest);
    }
    loop {
        auddevice.set_vol((volume * 100.0) as u8);
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
        if is_down_pressed() {
            selected_option = (selected_option + 1) % 4;
        };
        if is_up_pressed() {
            selected_option = (selected_option - 1) % 4;
        };
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

        match selected_option {
            0 => {
                if is_enter_pressed() {
                    process::exit(3);
                }
            }
            1 => {
                if is_left_pressed() {
                    volume = (volume - 0.1 as f64).max(0.0);
                }
                if is_right_pressed() {
                    volume = (volume + 0.1 as f64).min(1.0);
                }
            }
            2 => {
                if is_enter_pressed() {
                    shutdown().expect("Counldn't shutdown");
                    return;
                }
            }
            3 => {
                if is_enter_pressed() {
                    process::exit(4);
                }
            }
            _ => {}
        }
        let battery_tex = (battery_level * 6.0 as f64).round() as usize;
        let vol_tex = (volume * 10.0 as f64).round() as usize;
        draw_texture_ex(
            &volume_textures[vol_tex as usize],
            (53.0 / bg_texture.width()) * target_width,
            (64.0 / bg_texture.height()) * height,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    volume_textures[vol_tex as usize].width() * scale_factor,
                    volume_textures[vol_tex as usize].height() * scale_factor,
                )),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &battery_textures[battery_tex as usize],
            (17.0 / bg_texture.width()) * target_width,
            (106.0 / bg_texture.height()) * height,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    battery_textures[battery_tex as usize].width() * scale_factor,
                    battery_textures[battery_tex as usize].height() * scale_factor,
                )),
                ..Default::default()
            },
        );
        if ((battery_level * 100.0) as u32 / 10 == 10) {
            draw_digit(
                1,
                (50.0 / bg_texture.width()) * target_width,
                (113.0 / bg_texture.height()) * height,
                scale_factor,
                &textures[4],
            );
        }
        draw_digit(
            match (battery_level * 100.0) as u32 / 10 {
                0..9 => (battery_level * 100.0) as u32 / 10,
                _ => 0,
            },
            (67.0 / bg_texture.width()) * target_width,
            (113.0 / bg_texture.height()) * height,
            scale_factor,
            &textures[4],
        );
        draw_digit(
            (battery_level * 100.0) as u32 % 10,
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
fn is_up_pressed() -> bool {
    let mut gp = Gamepads::new();
    gp.poll();
    is_key_pressed(KeyCode::Up)
        || is_key_pressed(KeyCode::W)
        || gp.all().any(|g| g.is_currently_pressed(Button::DPadUp))
}
fn is_down_pressed() -> bool {
    let mut gp = Gamepads::new();
    gp.poll();
    is_key_pressed(KeyCode::Down)
        || is_key_pressed(KeyCode::S)
        || gp.all().any(|g| g.is_currently_pressed(Button::DPadDown))
}
fn is_left_pressed() -> bool {
    let mut gp = Gamepads::new();
    gp.poll();
    is_key_pressed(KeyCode::Left)
        || is_key_pressed(KeyCode::A)
        || gp.all().any(|g| g.is_currently_pressed(Button::DPadLeft))
}
fn is_right_pressed() -> bool {
    let mut gp = Gamepads::new();
    gp.poll();
    is_key_pressed(KeyCode::Right)
        || is_key_pressed(KeyCode::D)
        || gp.all().any(|g| g.is_currently_pressed(Button::DPadRight))
}
fn is_enter_pressed() -> bool {
    let mut gp = Gamepads::new();
    gp.poll();
    is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::Space)
        || gp.all().any(|g| g.is_currently_pressed(Button::ActionDown))
}
