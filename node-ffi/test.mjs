import { Robot, sleep, testThreadsafeFunction } from "./index.js"

testThreadsafeFunction((...x) => console.log(x));

console.log(Robot, sleep);

const robot = await Robot.connect("127.0.0.1:1110");
// const robot = await Robot.connect("10.0.0.9:1110");
console.log(robot);
console.log(await robot.getUptime());
console.log(await robot.readPin(3));
// for (let i = 0; true; i++) {
//     console.log(i, await robot.drive(((i % 25) + 25) / 100, -(i % 25) + 25) / 100);
//     await sleep(100);
// }
// robot.subscribe("GpioPin", 2, (...x) => console.log(x));
setTimeout(() => { }, 1000000000);
