# Spirit Level

Recently [xssfox posted](https://sprocketfox.io/xssfox/2021/12/02/xrandr/) that xrandr is able to do incredible things, for example render the image diagonally on the screen.

What if the screen could be updated dynamically based on a sensor data?

## Hardware

- monitor with pivot function
- microbit v2 board with lsm303agr sensor
- USB cable
- host PC running Fedora on Xorg

## Software

- Rust microcontroller code sending accelerometer readings over UARTE/USB
- Python listener computing xrandr transformations
- `xrandr` adjusting screen position

## How to run

### Microcontroller

Plug in the USB cable and flash the device.

```bash
$ cargo embed --target thumbv7em-none-eabihf
```

### Listener

Adjust `DEV` (your USB interface) AND `SCREEN` in `listener/src/main.py`.
Run `xrandr` to show available screens. 

You might also want to add yourself to dialout group to avoid calling the script with root permissions.

```bash
$ sudo usermod -a -G dialout $USER
```

```bash
cd listener
poetry install
poetry run python src/main.py
```
