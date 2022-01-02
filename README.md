# roblib

#### _a remove library for a dank engine._

this repo contains the new roblib server written in rust as well as a client
library also written in rust.

### client libraries in other languages:

-   [javascript/typescript](https://github.com/kareszklub/roblib-client)

## server api

the server commuinicates via standard websockets. the initial websocket endpoint
is `/ws`

the commands are plain text and have a format of:

```
command arg1 arg2 ...
```

**the available commands are:**

-   `m left right`: move the robot's left and right motors with the given speed
-   `s`: stop the robot
-   `l r g b`: set the leds to the given color, r,g,b can be 0 or 1
-   `v angle`: set the servo to the given absolute angle
-   `b freq`: sounds the buzzer at the given frequency
-   `t`: get the data from the four onboard sensors in the format of four comma
    separated numbers

as per the websocket standard, the client is expected to respond to pings.
failing to do so will result in the connection being closed.
