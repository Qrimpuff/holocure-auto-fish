use std::{
    thread,
    time::{Duration, Instant},
};

use enigo::*;
use image::RgbaImage;
use win_screenshot::prelude::*;

const WINDOW_TITLE: &str = "HoloCure";

// Key configuration
const FISHING_KEY: Key = Key::Space;
const ROUND_KEY: Key = Key::Space;
const UP_KEY: Key = Key::W;
const DOWN_KEY: Key = Key::S;
const LEFT_KEY: Key = Key::A;
const RIGHT_KEY: Key = Key::D;

// adjusted for 1920x1080
const CROP_TOP: i32 = 680;
const CROP_BOTTOM: i32 = 810;
const CROP_LEFT: i32 = 1130;
const CROP_RIGHT: i32 = 1230;

const PIXEL_DELTA: u8 = 10;
const FISHING_THRESHOLD: usize = 500;
const SHAPE_THRESHOLD: usize = 300;

#[derive(Debug, PartialEq)]
enum FishKey {
    Round,
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    println!("-- HoloCure need to be in windowed mode 1920x1080. --");

    let mut enigo = Enigo::new();
    let hwnd = find_window(WINDOW_TITLE).unwrap();

    let mut last_key: Option<FishKey> = None;
    let mut last_key_time: Option<Instant> = None;
    let mut fishing = false;
    let mut try_fishing_attempt = 0;
    let mut last_fishing_attempt = Instant::now();

    loop {
        let now = Instant::now();

        let buf = capture_window_ex(
            hwnd,
            Using::PrintWindow,
            Area::ClientOnly,
            Some([CROP_LEFT, CROP_TOP]),
            Some([CROP_RIGHT - CROP_LEFT, CROP_BOTTOM - CROP_TOP]),
        )
        .unwrap();

        // convert to image and save for debugging
        let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
        // img.save("screenshot.jpg").unwrap();

        if fishing {
            let key = key_to_press(&img);
            if let Some(ref key) = key {
                match key {
                    FishKey::Round => enigo.key_down(ROUND_KEY),
                    FishKey::Up => enigo.key_down(UP_KEY),
                    FishKey::Down => enigo.key_down(DOWN_KEY),
                    FishKey::Left => enigo.key_down(LEFT_KEY),
                    FishKey::Right => enigo.key_down(RIGHT_KEY),
                }
                if last_key.as_ref() != Some(key) {
                    println!("pressed {key:?}");
                }
                last_key_time = Some(Instant::now());
            } else {
                enigo.key_up(ROUND_KEY);
                enigo.key_up(UP_KEY);
                enigo.key_up(DOWN_KEY);
                enigo.key_up(LEFT_KEY);
                enigo.key_up(RIGHT_KEY);
                if let Some(last_key_time) = last_key_time {
                    if last_key_time.elapsed() > Duration::from_secs(1) {
                        fishing = false;
                        println!("stopped fishing!");
                    }
                } else if last_fishing_attempt.elapsed() > Duration::from_secs(10) {
                    fishing = false;
                    println!("stopped fishing! (too long)");
                }
            }
            last_key = key;
        } else if is_fishing(&img) {
            fishing = true;
            try_fishing_attempt = 0;
            last_key_time = None;
            last_fishing_attempt = Instant::now();
            println!("fishing!");
        } else {
            last_key_time = None;
            thread::sleep(Duration::from_millis(500));
            // try to fish
            if last_fishing_attempt.elapsed()
                > Duration::from_millis(500 * (try_fishing_attempt + 1))
            {
                println!("trying to fish...");
                enigo.key_click(FISHING_KEY);
                try_fishing_attempt += 1;
                last_fishing_attempt = Instant::now();
            }
        }

        thread::sleep(Duration::from_millis(10).saturating_sub(now.elapsed()));
    }
}

fn key_to_press(img: &RgbaImage) -> Option<FishKey> {
    if is_round(img) {
        return Some(FishKey::Round);
    }
    if is_up(img) {
        return Some(FishKey::Up);
    }
    if is_down(img) {
        return Some(FishKey::Down);
    }
    if is_left(img) {
        return Some(FishKey::Left);
    }
    if is_right(img) {
        return Some(FishKey::Right);
    }
    None
}

fn is_fishing(img: &RgbaImage) -> bool {
    let r = 251;
    let g = 251;
    let b = 251;
    is_shape(img, r, g, b, FISHING_THRESHOLD)
}
fn is_round(img: &RgbaImage) -> bool {
    let r = 171;
    let g = 52;
    let b = 206;
    is_shape(img, r, g, b, SHAPE_THRESHOLD)
}
fn is_up(img: &RgbaImage) -> bool {
    let r = 224;
    let g = 51;
    let b = 55;
    is_shape(img, r, g, b, SHAPE_THRESHOLD)
}
fn is_down(img: &RgbaImage) -> bool {
    let r = 52;
    let g = 145;
    let b = 247;
    is_shape(img, r, g, b, SHAPE_THRESHOLD)
}
fn is_left(img: &RgbaImage) -> bool {
    let r = 243;
    let g = 201;
    let b = 67;
    is_shape(img, r, g, b, SHAPE_THRESHOLD)
}
fn is_right(img: &RgbaImage) -> bool {
    let r = 41;
    let g = 231;
    let b = 43;
    is_shape(img, r, g, b, SHAPE_THRESHOLD)
}

fn is_shape(img: &RgbaImage, r: u8, g: u8, b: u8, threshold: usize) -> bool {
    img.pixels()
        .filter(|p| {
            p.0[0] >= r.saturating_sub(PIXEL_DELTA)
                && p.0[0] <= r.saturating_add(PIXEL_DELTA)
                && p.0[1] >= g.saturating_sub(PIXEL_DELTA)
                && p.0[1] <= g.saturating_add(PIXEL_DELTA)
                && p.0[2] >= b.saturating_sub(PIXEL_DELTA)
                && p.0[2] <= b.saturating_add(PIXEL_DELTA)
        })
        .count()
        > threshold
}
