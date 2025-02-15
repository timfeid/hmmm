import Phaser from "phaser";
import type { Actionable } from "./actionable";

export class Person implements Actionable {
  public id: string;
  public sprite: Phaser.Physics.Arcade.Sprite;
  public speed: number = 80;

  constructor(id: string, sprite: Phaser.Physics.Arcade.Sprite) {
    this.id = id;
    this.sprite = sprite;
  }

  getSprite(): Phaser.Physics.Arcade.Sprite {
    return this.sprite;
  }

  isActionable(uid: string) {
    console.log(uid, this.id);
    return uid === this.id;
  }

  action() {
    console.log("go back to the person pls");
  }

  updateInput(cursors: Phaser.Types.Input.Keyboard.CursorKeys, delta: number) {
    const dt = delta / 1000;
    let vx = 0,
      vy = 0;
    if (cursors.up.isDown) {
      vy = -this.speed;
    } else if (cursors.down.isDown) {
      vy = this.speed;
    }
    if (cursors.left.isDown) {
      vx = -this.speed;
    } else if (cursors.right.isDown) {
      vx = this.speed;
    }
    if (vx > 0 || vy > 0) {
      if (this.sprite.anims.currentAnim?.key !== "walk") {
        this.sprite.anims.play("walk");
      }
    } else {
      if (this.sprite.anims.currentAnim?.key !== "idle") {
        this.sprite.anims.play("idle");
      }
    }
    this.sprite.body.setVelocity(vx, vy);
  }

  getInputState(): InputState {
    return {
      up: false,
      down: false,
      left: false,
      right: false,
      rotation: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
    };
  }
}
