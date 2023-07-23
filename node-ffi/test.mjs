import { Robot, sleep } from "./index.js"

console.log(Robot, sleep);

const robot = new Robot("localhost:1110");
// console.log(robot)
// console.log(robot.getUptime())
for (let i = 0; true; i++) {
    console.log(i, robot.drive(i % 100, -i % 100));
    sleep(100);
}
