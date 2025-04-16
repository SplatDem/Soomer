use libwayshot::WayshotConnection;
use sdl2::{
    event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color, rect::Rect,
    render::TextureCreator, video::WindowContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{time::Duration, fs};

const _SPOTLIGHT_TINT: Color = Color::RGBA(0, 0, 0, 190); // TODO: read shader path from config or
                                                          // use default

#[derive(Debug, Deserialize, Serialize)]
struct ConfigBgColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Serialize, Deserialize)]
struct Config {
    bg: ConfigBgColor,
    max_scale: f32,
    min_scale: f32,
}

struct Display {
    texture_x: f32,
    texture_y: f32,
    dragging: bool,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    mouse_x: i32,
    mouse_y: i32,
}

impl Display {
    fn new() -> Self {
	Display {
	    texture_x: 0.0,
	    texture_y: 0.0,
	    dragging: false,
	    offset_x: 0.0,
	    offset_y: 0.0,
	    scale: 1.0,
	    mouse_x: 0,
	    mouse_y: 0,
	}
    }

    fn reset(&mut self) {
	self.texture_x = 0.0;
        self.texture_y = 0.0;
        self.dragging = false;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.scale = 1.0;
    }
}

fn load_config() -> Result<Config> {
    let mut config_path = std::path::PathBuf::from(std::env::var("HOME").unwrap());
    config_path.push(".config/soomer/config.json");

    if !config_path.exists() {
        eprintln!("WARNING: Config not found at {}", config_path.display());
        println!("NOTE: Creating default config at {}...", config_path.display());
        
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
            max_scale: 10.0,
            min_scale: 0.1,
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
    let wayshot_connection =
        WayshotConnection::new().expect("ERROR: failed to connect to the wayland display server");
    let screenshot_image = wayshot_connection
        .screenshot_all(false)
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
    
    let config = load_config().expect("ERROR: Failed to load config");
    
    let mut display = Display::new();
    
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'running Ok(()),
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
                        display.texture_x = x as f32 - display.offset_x;
                        display.texture_y = y as f32 - display.offset_y;
                    }
                }
                Event::MouseWheel { y, .. } => {
                    let old_scale = display.scale;
                    
                    if y > 0 {
			                  display.scale = (display.scale * 1.1f32).min(config.max_scale);
                    } else if y < 0 {
                        display.scale = (display.scale / 1.1f32).max(config.min_scale);
                    }
                    
                    let rel_x = (display.mouse_x as f32 - display.texture_x) / old_scale;
                    let rel_y = (display.mouse_y as f32 - display.texture_y) / old_scale;
                    
                    display.texture_x = display.mouse_x as f32 - rel_x * display.scale;
                    display.texture_y = display.mouse_y as f32 - rel_y * display.scale;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R), // Reset
                    ..
                } => { display.reset(); }
                Event::KeyDown {
                    keycode: Some(Keycode::S), // Scale reset
                    ..
                } => {
                    let center_x = display.texture_x + (width as f32 * display.scale) / 2.0;
                    let center_y = display.texture_y + (height as f32 * display.scale) / 2.0;
                    display.scale = 1.0;
                    display.texture_x = center_x - width as f32 / 2.0;
                    display.texture_y = center_y - height as f32 / 2.0;
                }
                _ => {}
            }
        }

	canvas.set_draw_color(Color::RGBA(config.bg.r, config.bg.g, config.bg.b, config.bg.a));
        canvas.clear();
        
        let tex_width = (width as f32 * display.scale) as u32;
        let tex_height = (height as f32 * display.scale) as u32;
        
        if tex_width < width {
            display.texture_x = (width as f32 - tex_width as f32) / 2.0;
        }
        if tex_height < height {
            display.texture_y = (height as f32 - tex_height as f32) / 2.0;
        }
        
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
        std::thread::sleep(Duration::from_millis(16));
    }
}
