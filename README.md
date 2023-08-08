# roblib

#### _a remote library for a dank engine._

this repo contains the new roblib server and client library written in rust,
as well as bindings to said library for nodejs. the server has two versions:
**base**, that only accepts gpio and *(software)* pwm commands,
and **roland**, a specialized version for the dank engine.

## client libraries in other languages:

-   [javascript/typescript browser](https://github.com/kareszklub/roblib-client) *(wip: commands and returns work, events don't)*

## server downloads

*the cache may be old. to bypass it, open the lates commit and use the links in the comments*

[View all](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main)
| <!-- empty -->                 | base                                                                                                                                | roland                                                                                                                          |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| aarch64-unknown-linux-gnu      | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-aarch64-unknown-linux-gnu.zip)      | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-aarch64-unknown-linux-gnu.zip)      |
| aarch64-unknown-linux-musl     | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-aarch64-unknown-linux-musl.zip)     | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-aarch64-unknown-linux-musl.zip)     |
| armv7-unknown-linux-gnueabihf  | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-armv7-unknown-linux-gnueabihf.zip)  | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-armv7-unknown-linux-gnueabihf.zip)  |
| armv7-unknown-linux-musleabihf | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-armv7-unknown-linux-musleabihf.zip) | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-armv7-unknown-linux-musleabihf.zip) |
| x86_64-pc-windows-msvc         | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-x86_64-pc-windows-msvc.zip)         | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-x86_64-pc-windows-msvc.zip)         |
| x86_64-unknown-linux-gnu       | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-x86_64-unknown-linux-gnu.zip)       | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-x86_64-unknown-linux-gnu.zip)       |
| x86_64-unknown-linux-musl      | [Download](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-base-x86_64-unknown-linux-musl.zip)      | [Download](https://nightly.link/kareszklub/roblib-rs/actions/runs/5766762992/roblib-server-roland-x86_64-unknown-linux-musl.zip)      |

## server api

| Transport | Binary | Text | Port | Additional info |
| --------- | ------ | ---- | ---- | --------------- |
| TCP       | Yes    | No   | 1110 |                 |
| UDP       | Yes    | No   | 1110 |                 |
| WebSocket | Yes    | Yes  | 1111 | Endpoint: /ws   |
| HTTP POST | No     | Yes  | 1111 | Endpoint: /cmd  |

# Binary format

The binary format is using [bincode](https://lib.rs/bincode).

Each transport implements its own wire format that aligns with its advantages.

It isnt't guaranteed to be stable, implementing it outside of here isn't recommended.

# Text format

The text format is designed to be simpler.

It consists of an ID *(u32)*, the command prefix, and any additional arguments for that command,
separated by a space.


```
1 command arg1 arg2 ...
```

## Available commands and their arguments

### Built-in

These require no additional feature flags and are always available.

- `+ event eventargs`: Subscribe to an event
- `- event eventargs`: Unsubscribe from an event
- `0`: No-op
- `U`: Get server uptime in seconds
- `X`: Abort: immediately perform a clean shutdown

### GPIO

These commands are designed to control generic gpio and (software) pwm devices.

- `p pin mode`: REQUIRED! Set a pin to input or output mode
- `r pin`: Read a pin's logic level
- `w pin logic`: Write a pin to high or low, logic can be 0 or 1
- `W pin hz cycle`: Configure a software pwm on a pin, hz is the frequency, and
cycle is the duty cycle, a number between 0 and 100
- `V pin deg`: Move a servo between -90 and 90 degrees, using PWM

### Roland

The dank engine.

-   `m left right`: Move the robot's left and right motors with the given speed (between 0 and 1)
-   `M aleft aright`: Move the robot by specifying two angles for the two motors
-   `s`: Stop the robot
-   `l r g b`: Set the leds to the given color, r,g,b can be 0 or 1
-   `a angle`: Set the servo to the given absolute angle
-   `b freq`: Sounds the buzzer at the given frequency
-   `t`: Get the data from the four onboard sensors in the format of four comma
    separated boolean numbers
-   `u`: Read the onboard ultra sensor, returns the distance in meters

### Camloc

[Camera location service](https://github.com/Kris030/camloc)

- `P`: Get the position of the robot

## Profiles

We currently have two profiles, for the two primary use-cases.

- **Base**: Includes the GPIO features, for a generic gpio pin controller
- **Roland**: Includes everything, for the dank engine
