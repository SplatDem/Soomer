# Soomer
Soomer is a zoomer application for wayland written in rust.

![demo](demo.gif)

## Bindings
| Key   | Action      |
|-------|-------------|
| `ESC` | Quit        |
| `S`   | Reset scale |
| `R`   | Reset all   |

## Configuration
Default config:
```json
{
    "bg": {
        "r": 10,
        "g": 0,
        "b": 15,
        "a": 255
    },
    "scale": {
	"max_scale": 10.0,
	"min_scale": 1.0,
	"scale_factor": 1.5
    },
    "update_delay": 60
}
```

## Building
```
cargo build --release
sudo cp target/release/soomer /bin
```
