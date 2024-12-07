use std::{
    cmp::Ordering,
    env, thread,
    time::{Duration, Instant},
};

use enigo::*;
use image::{imageops::crop_imm, RgbaImage};
use win_screenshot::prelude::*;

const WINDOW_TITLE: &str = "HoloCure";

// Key configuration
const GATHERING_KEY: Key = Key::Space;
const LEFT_KEY: Key = Key::A;
const RIGHT_KEY: Key = Key::D;

// adjusted for 1920x1080
// const WINDOW_WIDTH: i32 = 1920;
// const WINDOW_HEIGHT: i32 = 1080;

const CROP_TOP: i32 = 820;
const CROP_BOTTOM: i32 = 890;
const CROP_LEFT: i32 = 615;
const CROP_RIGHT: i32 = 1330;

const ARROW_HALF_WIDTH: u32 = 20;
const IS_GATHERING_X1: u32 = 0;
const IS_GATHERING_X2: u32 = 650;
const IS_GATHERING_WIDTH: u32 = 35;
const IS_GATHERING_HEIGHT: u32 = CROP_BOTTOM as u32 - CROP_TOP as u32;

const PIXEL_DELTA: u8 = 10;
const GATHERING_THRESHOLD: usize = 400;
const HIT_THRESHOLD: usize = 300;

pub fn start_gathering() {
    println!("-- Gathering --");

    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let hwnd = find_window(WINDOW_TITLE).unwrap();

    let mut last_key_time: Option<Instant> = None;
    let mut gathering = false;
    let mut gathering_range = (0, 0);
    let mut gathering_base_threshold = 0;
    let mut last_gathering_attempt = Instant::now();
    let mut last_direction_change = Instant::now();
    let mut go_left = false;

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
        // img.save("screenshot.png").unwrap();

        if gathering {
            let range = crop_imm(
                &img,
                gathering_range.0 - ARROW_HALF_WIDTH,
                0,
                gathering_range.1 - gathering_range.0 + ARROW_HALF_WIDTH + ARROW_HALF_WIDTH,
                IS_GATHERING_HEIGHT,
            );
            // range.to_image().save("screenshot.png").unwrap();

            // hit
            // white-ish
            let r = 251;
            let g = 251;
            let b = 251;
            if is_shape(
                &range.to_image(),
                r,
                g,
                b,
                gathering_base_threshold + HIT_THRESHOLD,
            ) {
                enigo.key(GATHERING_KEY, Direction::Press).unwrap();
                thread::sleep(Duration::from_millis(50));
                enigo.key(GATHERING_KEY, Direction::Release).unwrap();
                println!("hit!");
                last_key_time = Some(Instant::now());
                thread::sleep(Duration::from_millis(400));
            }

            if let Some(last_key_time) = last_key_time {
                if last_key_time.elapsed() > Duration::from_millis(500) {
                    gathering = false;
                    println!("stopped gathering!");
                    thread::sleep(Duration::from_millis(600));
                    enigo.key(GATHERING_KEY, Direction::Press).unwrap();
                    thread::sleep(Duration::from_millis(50));
                    enigo.key(GATHERING_KEY, Direction::Release).unwrap();
                }
            } else if last_gathering_attempt.elapsed() > Duration::from_secs(1) {
                gathering = false;
                println!("stopped gathering! (too long)");
                thread::sleep(Duration::from_millis(600));
                enigo.key(GATHERING_KEY, Direction::Press).unwrap();
                thread::sleep(Duration::from_millis(50));
                enigo.key(GATHERING_KEY, Direction::Release).unwrap();
            }
        } else if is_gathering(&img) {
            gathering = true;
            gathering_range = find_range(&img);
            let range = crop_imm(
                &img,
                gathering_range.0 - ARROW_HALF_WIDTH,
                0,
                gathering_range.1 - gathering_range.0 + ARROW_HALF_WIDTH + ARROW_HALF_WIDTH,
                IS_GATHERING_HEIGHT,
            );
            // range.to_image().save("screenshot.png").unwrap();
            // white-ish
            let r = 251;
            let g = 251;
            let b = 251;
            gathering_base_threshold = count_threshold(&range.to_image(), r, g, b);
            //     try_gathering_attempt = 0;
            last_key_time = None;
            last_gathering_attempt = Instant::now();
            println!("gathering!");
            enigo.key(LEFT_KEY, Direction::Release).unwrap();
            enigo.key(RIGHT_KEY, Direction::Release).unwrap();
        } else {
            last_key_time = None;
            thread::sleep(Duration::from_millis(500));
            // try to gather
            if last_gathering_attempt.elapsed() > Duration::from_millis(500) {
                println!("trying to gather...");
                enigo.key(GATHERING_KEY, Direction::Press).unwrap();
                thread::sleep(Duration::from_millis(100));
                enigo.key(GATHERING_KEY, Direction::Release).unwrap();
                last_gathering_attempt = Instant::now();
            }

            if go_left {
                enigo.key(RIGHT_KEY, Direction::Release).unwrap();
                enigo.key(LEFT_KEY, Direction::Press).unwrap();
            } else {
                enigo.key(LEFT_KEY, Direction::Release).unwrap();
                enigo.key(RIGHT_KEY, Direction::Press).unwrap();
            }
            if last_direction_change.elapsed() > Duration::from_secs(5) {
                println!("changing direction...");
                go_left = !go_left;
                last_direction_change = Instant::now();
            }
        }

        thread::sleep(Duration::from_millis(10).saturating_sub(now.elapsed()));
    }
}

fn is_gathering(img: &RgbaImage) -> bool {
    let arrow1 = crop_imm(
        img,
        IS_GATHERING_X1,
        0,
        IS_GATHERING_WIDTH,
        IS_GATHERING_HEIGHT,
    );
    let arrow2 = crop_imm(
        img,
        IS_GATHERING_X2,
        0,
        IS_GATHERING_WIDTH,
        IS_GATHERING_HEIGHT,
    );

    // white-ish
    let r = 251;
    let g = 251;
    let b = 251;
    is_shape(&arrow1.to_image(), r, g, b, GATHERING_THRESHOLD)
        && is_shape(&arrow2.to_image(), r, g, b, GATHERING_THRESHOLD)
}

fn find_range(img: &RgbaImage) -> (u32, u32) {
    let count_threshold = 10;

    let mut min = (u32::MAX, 0);
    let mut prev_min = (u32::MAX, 0);
    let mut max = (u32::MIN, 0);
    let mut prev_max = (u32::MIN, 0);

    // red-ish
    let r: u8 = 251;
    let g: u8 = 0;
    let b: u8 = 0;

    for (x, _y, p) in img.enumerate_pixels() {
        if p.0[0] >= r.saturating_sub(PIXEL_DELTA)
            && p.0[0] <= r.saturating_add(PIXEL_DELTA)
            && p.0[1] >= g.saturating_sub(PIXEL_DELTA)
            && p.0[1] <= g.saturating_add(PIXEL_DELTA)
            && p.0[2] >= b.saturating_sub(PIXEL_DELTA)
            && p.0[2] <= b.saturating_add(PIXEL_DELTA)
        {
            match x.cmp(&min.0) {
                Ordering::Less => {
                    if min.1 >= count_threshold {
                        prev_min = min;
                    }
                    min.0 = x;
                    min.1 = 1;
                }
                Ordering::Equal => {
                    min.1 += 1;
                }
                Ordering::Greater => {}
            }
            match x.cmp(&max.0) {
                Ordering::Greater => {
                    if max.1 >= count_threshold {
                        prev_max = max;
                    }
                    max.0 = x;
                    max.1 = 1;
                }
                Ordering::Equal => {
                    max.1 += 1;
                }
                Ordering::Less => {}
            }
        }
    }

    if min.1 < count_threshold {
        min = prev_min;
    }
    if max.1 < count_threshold {
        max = prev_max;
    }

    (min.0, max.0)
}

fn is_shape(img: &RgbaImage, r: u8, g: u8, b: u8, threshold: usize) -> bool {
    count_threshold(img, r, g, b) > threshold
}

fn count_threshold(img: &RgbaImage, r: u8, g: u8, b: u8) -> usize {
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
}
