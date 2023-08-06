/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export function sum(a: number, b: number): number
export function sleepBlocking(ms: number): void
export function sleep(ms: number): Promise<void>
export function throws(n: number): Promise<void>
export const enum EventType {
  TrackSensor = 'TrackSensor',
  UltraSensor = 'UltraSensor',
  GpioPin = 'GpioPin',
  CamlocConnect = 'CamlocConnect',
  CamlocDisconnect = 'CamlocDisconnect',
  CamlocPosition = 'CamlocPosition',
  CamlocInfoUpdate = 'CamlocInfoUpdate'
}
export const enum PinMode {
  input = 'input',
  output = 'output'
}
export interface Position {
  x: number
  y: number
  rotation: number
}
export type JsRobot = Robot
export class Robot {
  constructor()
  static connect(addr: string): Promise<Robot>
  disconnect(): void
  nop(): Promise<void>
  getUptime(): Promise<number>
  drive(left: number, right: number): Promise<void>
  led(r: boolean, g: boolean, b: boolean): Promise<void>
  rolandServo(degree: number): Promise<void>
  buzzer(pw: number): Promise<void>
  trackSensor(): Promise<boolean[]>
  ultraSensor(): Promise<number>
  readPin(pin: number): Promise<boolean>
  writePin(pin: number, value: boolean): Promise<void>
  pwm(pin: number, hz: number, cycle: number): Promise<void>
  servo(pin: number, degree: number): Promise<void>
  pinMode(pin: number, mode: JsPinMode): Promise<void>
  getPosition(): Promise<JsPosition | null>
  subscribe(ev: JsEventType, evArgs: any, handler: (...args: any[]) => any): void
}
