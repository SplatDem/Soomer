use image::{ImageBuffer, RgbaImage};
use libwayshot::WayshotConnection;
use sdl2::{
    event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color, rect::Rect,
    render::TextureCreator, video::WindowContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::time::{self, Duration};

#[derive(Deserialize, Serialize)]
struct ConfigBgColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Serialize, Deserialize)]
struct ConfigScale {
    max: f32,
    min: f32,
    factor: f32,
}

#[derive(Serialize, Deserialize)]
struct Config {
    bg: ConfigBgColor,
    scale: ConfigScale,
    smooth_factor: f32,
    update_delay: u64,
    monitor: usize,
    screenshot_save_path: String,
    screenshot_save_name: String,
}

struct Display {
    texture_x: f32,
    texture_y: f32,
    target_texture_x: f32,
    target_texture_y: f32,
    dragging: bool,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    target_scale: f32,
    mouse_x: i32,
    mouse_y: i32,
}

impl Display {
    fn new() -> Self {
        Display {
            texture_x: 0.0,
            texture_y: 0.0,
            target_texture_x: 0.0,
            target_texture_y: 0.0,
            dragging: false,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            target_scale: 1.0,
            mouse_x: 0,
            mouse_y: 0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn reset_scale(&mut self, width: u32, height: u32) {
        let center_x = self.target_texture_x + (width as f32 * self.scale) / 2.0;
        let center_y = self.target_texture_y + (height as f32 * self.scale) / 2.0;
        self.target_scale = 1.0;
        self.target_texture_x = center_x - width as f32 / 2.0;
        self.target_texture_y = center_y - height as f32 / 2.0;
    }

    fn smooth_update(&mut self, smooth_factor: f32) {
        self.scale = lerp(self.scale, self.target_scale, smooth_factor);
        self.texture_x = lerp(self.texture_x, self.target_texture_x, smooth_factor);
        self.texture_y = lerp(self.texture_y, self.target_texture_y, smooth_factor);
    }

    fn save_screenshot(&mut self, image_to_save: RgbaImage, save_path: &str, save_name: &str) {
        let save_path = format!(
            "{}/smr_{:?}_{}",
            save_path,
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .expect("ERROR: You've made a fucking time machine"),
            save_name
        );
        image_to_save
            .save(save_path)
            .expect("ERROR: Failed to save screenshot");
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn load_config() -> Result<Config> {
    let mut config_path = std::path::PathBuf::from(std::env::var("HOME").unwrap());
    config_path.push(".config/soomer.json");

    if !config_path.exists() {
        println!("WARNING: Config not found at {}", config_path.display());
        println!(
            "NOTE: Creating default config at {}...",
            config_path.display()
        );

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("ERROR: Failed to create parent directories");
        }

        let default_config = Config {
            bg: ConfigBgColor {
                r: 10,
                g: 0,
                b: 15,
                a: 255,
            },
            scale: ConfigScale {
                max: 10.0,
                min: 0.1,
                factor: 1.1,
            },
            update_delay: 60,
            smooth_factor: 0.15,
            monitor: 0,
            screenshot_save_path: "./".to_string(),
            screenshot_save_name: "screenshot.png".to_string(),
        };

        let config_data = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, config_data).expect("ERROR: Failed to write config data");

        println!("NOTE: Default config written at {}", config_path.display());
    }

    let config_data = fs::read_to_string(&config_path).expect("ERROR: Failed to load config data");
    let config: Config = serde_json::from_str(&config_data)?;

    Ok(config)
}

fn main() -> Result<()> {
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            println!("ERROR: Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let wayshot_connection =
        WayshotConnection::new().expect("ERROR: failed to connect to the wayland display server");

    let outputs_info = WayshotConnection::get_all_outputs(&wayshot_connection);
    
    let screenshot_image = wayshot_connection
        // .screenshot_all(false)
	.screenshot_single_output(&outputs_info[config.monitor], true)
        .expect("ERROR: failed to take a screenshot")
        .to_rgba8();
    let (width, height) = screenshot_image.dimensions();

    let sdl_context = sdl2::init().expect("ERROR: Failed to initialize SDL");
    let video_subsystem = sdl_context
        .video()
        .expect("ERROR: Failed to get video subsystem");

    let window = video_subsystem
        .window("Soomer", width, height)
        .position_centered()
        .fullscreen()
        .always_on_top()
        .allow_highdpi()
        .build()
        .expect("ERROR: Failed to create window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("ERROR: Failed to create canvas");

    let texture_creator: TextureCreator<WindowContext> = canvas.texture_creator();

    let mut surface =
        sdl2::surface::Surface::new(width, height, sdl2::pixels::PixelFormatEnum::RGBA32)
            .expect("ERROR: Failed to create surface");

    surface.with_lock_mut(|pixels| {
        let image_data = screenshot_image.as_raw();
        pixels.copy_from_slice(image_data);
    });

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .expect("ERROR: Failed to create texture");

    let mut events = sdl_context.event_pump().unwrap();

    let mut display = Display::new();

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running Ok(()),
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'running Ok(()),
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        display.dragging = true;
                        display.offset_x = x as f32 - display.texture_x;
                        display.offset_y = y as f32 - display.texture_y;
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Left {
                        display.dragging = false;
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    display.mouse_x = x;
                    display.mouse_y = y;
                    if display.dragging {
                        display.target_texture_x = x as f32 - display.offset_x;
                        display.target_texture_y = y as f32 - display.offset_y;
                    }
                }
                Event::MouseWheel { y, .. } => {
                    let mouse_x = display.mouse_x as f32;
                    let mouse_y = display.mouse_y as f32;

                    let old_rel_x = (mouse_x - display.target_texture_x) / display.target_scale;
                    let old_rel_y = (mouse_y - display.target_texture_y) / display.target_scale;

                    if y > 0 {
                        display.target_scale *= config.scale.factor;
                    } else if y < 0 {
                        display.target_scale /= config.scale.factor;
                    }
                    display.target_scale = display
                        .target_scale
                        .clamp(config.scale.min, config.scale.max);

                    display.target_texture_x = mouse_x - old_rel_x * display.target_scale;
                    display.target_texture_y = mouse_y - old_rel_y * display.target_scale;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R), // Reset
                    ..
                } => {
                    display.reset();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C), // Scale reset
                    ..
                } => {
                    display.reset_scale(width, height);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    let img_to_save: RgbaImage =
                        ImageBuffer::from_raw(width, height, screenshot_image.clone().into_raw())
                            .unwrap();
                    display.save_screenshot(
                        img_to_save,
                        &config.screenshot_save_path,
                        &config.screenshot_save_name,
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => {
                    let new_screenshot_image = wayshot_connection
                        .screenshot_all(false)
                        .expect("ERROR: failed to take a screenshot")
                        .to_rgba8();
                    let (new_width, new_height) = screenshot_image.dimensions();
                    let img_to_save: RgbaImage = ImageBuffer::from_raw(
                        new_width,
                        new_height,
                        new_screenshot_image.into_raw(),
                    )
                    .unwrap();
                    display.save_screenshot(
                        img_to_save,
                        &config.screenshot_save_path,
                        &config.screenshot_save_name,
                    );
                }
                _ => {}
            }
        }

        display.smooth_update(config.smooth_factor);

        canvas.set_draw_color(Color::RGBA(
            config.bg.r,
            config.bg.g,
            config.bg.b,
            config.bg.a,
        ));
        canvas.clear();

        let tex_width = (width as f32 * display.scale) as u32;
        let tex_height = (height as f32 * display.scale) as u32;

        canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(
                    display.texture_x as i32,
                    display.texture_y as i32,
                    tex_width,
                    tex_height,
                )),
            )
            .expect("ERROR: Failed to copy texture");

        canvas.present();
        std::thread::sleep(Duration::from_millis(config.update_delay / 4));
    }
}
