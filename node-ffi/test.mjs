import { Robot, sleep, EventType } from "./index.js"

console.log(Robot, sleep);

const IN = 2, OUT = 3;

// const robot = await Robot.connect("localhost:1110");
const robot = await Robot.connect("roland:1110");
console.log(robot);
// await robot.pinMode(IN, "input");
// await robot.pinMode(OUT, "output");
// console.log(await robot.readPin(IN));
// robot.disconnect();

await roland_sensor();


// robot.subscribe("GpioPin", IN, (...x) => console.log(x));

// await uptime_loop();
// try { await drive_loop(); } catch (e) {
//     console.error(e);
//     process.exit(1);
// }

async function drive_loop() {
    for (let i = 0; true; i++) {
        const speed = ((i % 25) + 25) / 100;
        console.log(i, await robot.drive(speed, -speed));
        await sleep(100);
    }
}

async function uptime_loop() {
    while (true) {
        console.log(await robot.getUptime());
        await sleep(1000);
    }
}

async function roland_sensor() {
    robot.subscribe(EventType.TrackSensor, null, (_err, v) => console.log(v));
}

// setTimeout(() => { }, 1000000000);
