# roblib

## v2 TODO

- [x] efficient binary format
- [x] typesafe event system
- [x] camloc integration
- [x] extensively test gpio impl irl
- [ ] structured errors

- [x] server: tcp
- [x] server: websocket
- [x] server: udp
- [x] server: http

- [ ] rs client: async tcp
- [ ] rs client: async udp
- [x] rs client: sync tcp
- [x] rs client: sync udp
- [x] rs client: async http
- [x] rs client: async ws

- [ ] node client: async tcp
- [ ] node client: async udp
- [ ] node client: sync tcp
- [ ] node client: sync udp

- [ ] web client: async websocket
- [ ] web client: async http

*the rest of this readme is outdated at the moment, will be updated after the rewrite*

#### _a remote library for a dank engine._

this repo contains the new roblib server written in rust as well as a client
library for rust. the server has two versions: the generic, that only accepts
gpio and *(software)* pwm commands, and a specialized version for the dank engine.

### client libraries in other languages:

-   [javascript/typescript](https://github.com/kareszklub/roblib-client)

## server downloads

|             	| roblib-server                                                                                                       	| roland-server                                                                                                       	|
|-------------	|---------------------------------------------------------------------------------------------------------------------	|---------------------------------------------------------------------------------------------------------------------	|
| linux arm32 	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-armv7-unknown-linux-gnueabihf.zip) 	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roland-server-armv7-unknown-linux-gnueabihf.zip) 	|
| linux arm64 	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-aarch64-unknown-linux-gnu.zip)     	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roland-server-aarch64-unknown-linux-gnu.zip)     	|
| linux x64   	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-x86_64-unknown-linux-gnu.zip)      	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roland-server-x86_64-unknown-linux-gnu.zip)      	|
| windows x64 	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roblib-server-x86_64-pc-windows-msvc.zip)        	| [link](https://nightly.link/kareszklub/roblib-rs/workflows/ci/main/roland-server-x86_64-pc-windows-msvc.zip)        	|

## server api

the server communicates via standard websockets. the initial websocket endpoint
is `/ws`

the commands are plain text and have a format of:

```
command arg1 arg2 ...
```

**the available commands and their arguments:**

- `p pin logic`: set a pin to high or low, logic can be 0 or 1
- `w pin hz cycle`: configure a software pwm on a pin, hz is the frequency, and
cycle is the duty cycle, a number between 0 and 100

**roland edition:**

-   `m left right`: move the robot's left and right motors with the given speed
-   `s`: stop the robot
-   `l r g b`: set the leds to the given color, r,g,b can be 0 or 1
-   `v angle`: set the servo to the given absolute angle
-   `b freq`: sounds the buzzer at the given frequency
-   `t`: get the data from the four onboard sensors in the format of four comma
    separated numbers

as per the websocket standard, the client is expected to respond to pings.
failing to do so will result in the connection being closed.
