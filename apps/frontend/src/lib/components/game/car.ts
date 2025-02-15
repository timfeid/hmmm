// Car.ts
import Phaser from "phaser";
import type { Controllable, InputState } from "./controllable.js";
import type { Actionable } from "./actionable.js";

export class Car implements Controllable, Actionable {
  public id: string;
  public sprite: Phaser.Physics.Arcade.Sprite;
  public speed: number;
  public rotationSpeed: number;
  private targetState?: InputState;

  constructor(
    id: string,
    sprite: Phaser.Physics.Arcade.Sprite,
    speed = 100,
    rotationSpeed = 3
  ) {
    this.id = id;
    this.sprite = sprite;
    this.speed = speed;
    this.rotationSpeed = rotationSpeed;
  }

  isActionable(userId: string): boolean {
    return true;
  }

  action() {}

  getSprite(): Phaser.Physics.Arcade.Sprite {
    return this.sprite;
  }

  updateInput(cursors: Phaser.Types.Input.Keyboard.CursorKeys, delta: number) {
    const dt = delta / 1000;
    let desiredVX = 0;
    let desiredVY = 0;
    // Rotate using A and D:
    if (cursors.left.isDown) {
      this.sprite.rotation -= this.rotationSpeed * dt;
    } else if (cursors.right.isDown) {
      this.sprite.rotation += this.rotationSpeed * dt;
    }
    // Move forward/backward using W and S.
    const direction = this.sprite.rotation - Math.PI / 2;
    if (cursors.up.isDown) {
      desiredVX = Math.cos(direction) * this.speed;
      desiredVY = Math.sin(direction) * this.speed;
      this.sprite.setTexture("car-north");
    } else if (cursors.down.isDown) {
      desiredVX = -Math.cos(direction) * this.speed;
      desiredVY = -Math.sin(direction) * this.speed;
      this.sprite.setTexture("car-south");
    }
    // Update the physics body velocity if needed.
    this.sprite.body.setVelocity(desiredVX, desiredVY);
  }

  getInputState(): InputState {
    return {
      up: false, // Input state is not maintained internally by the Car,
      down: false, // but you could combine the car's current state with input.
      left: false,
      right: false,
      rotation: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
    };
  }
}
