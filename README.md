> [!WARNING]
> This program in the early stage of development
> Some features may not work
> Also there is a bug that prevents the program from being used in some DEs or compositors

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
        "max": 10.0,
        "min": 1.0,
        "factor": 1.5
    },
    "update_delay": 60,
    "smooth_factor": 0.1
}
```

## Building
```
cargo build --release
sudo cp target/release/soomer /bin
```

This program uses ZwlrScreencopy Manager, so it will not work in some DEs or compositors
![protocol](protocol.jpg)
Image from https://wayland.app/protocols/wlr-screencopy-unstable-v1
