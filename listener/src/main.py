import math
import serial
import subprocess

BAUD_RATE = 115200
DEV = "/dev/ttyACM0"
SCREEN = "HDMI-2"


def from_raw(bytearr: bytes) -> int:
    return int.from_bytes(bytearr, byteorder="big", signed=True)


def make_transform_arg(angle_rad: float) -> str:
    def fmt(num: float) -> str:
        return "{:.4f}".format(num)

    a = math.cos(angle_rad)
    b = -math.sin(angle_rad)
    # "Heuristic" adjustment that shifts screen to the left.
    # Works fine enough in my resolution (2560x1440).
    c = angle_rad * 1000
    d = math.sin(angle_rad)
    e = math.cos(angle_rad)
    f = 0
    return f"{fmt(a)},{fmt(b)},{c},{fmt(d)},{fmt(e)},{f},0,0,1"


def call_xrandr(arg: str):
    xrandr_argv = ["xrandr", "--output", SCREEN, "--transform", arg]
    print(" ".join(xrandr_argv))
    subprocess.check_call(xrandr_argv)


def main():
    with serial.Serial(DEV, BAUD_RATE, timeout=1) as ser:
        angle_vec = [0.1]
        call = True
        while True:
            line = ser.readline()
            # Ignore the last line break byte.
            line = line[0:-1]
            try:
                (x_raw, y_raw, z_raw) = line.split(b",")
                x = from_raw(x_raw)
                y = from_raw(y_raw)
                z = from_raw(z_raw)
                angle = math.atan2(x, y) + 0.00001  # Poor man's epsilon

                # Accelerometer produces some noise and xrandr would be called in excess,
                # that's why we keep history of angle changes.
                prev_angle = sum(angle_vec) / len(angle_vec)

                print(f"Angle: {angle}, {prev_angle}, ({x}, {y}, {z})")
                prop = angle / prev_angle
                if (prop > 1.2) or (prop < 0.8) and abs(prev_angle - angle) > 0.05:
                    transform_arg = make_transform_arg(angle)

                    # Sensor can't send data slower than 1 Hz,
                    # so we skip every other iteration.
                    # Sleeping in the main thread is not a good idea,
                    # because bytes in the serial port are buffered.
                    if call:
                        call_xrandr(transform_arg)
                        call = False
                    else:
                        call = True

                angle_vec.append(angle)
                if len(angle_vec) > 3:
                    angle_vec = angle_vec[1:]
            except ValueError:
                pass


if __name__ == "__main__":
    main()
