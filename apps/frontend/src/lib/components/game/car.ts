import Phaser from "phaser";
import type { Actionable, InputState } from "./actionable.js";
import type { PlayerController } from "./player-controller.js";
import type { ServerUpdatable } from "./updatable.js";
import type { CarObject } from "./utils.js";
import { user } from "../../stores/access-token.svelte.js";
import type { Controllable } from "./controllable.js";

export class Car implements Controllable, ServerUpdatable {
  id: string;
  public sprite: Phaser.Physics.Arcade.Sprite;
  public maxSpeed: number;
  public currentSpeed: number = 0;
  public acceleration: number;
  public breaking: number = 250;
  public deceleration: number;
  public rotationSpeed: number;
  public expectedUpdateInterval: number = 64;
  public lastServerUpdateTime: number = 0;
  private targetState?: InputState;
  private isBumping: boolean = false;
  private readonly roadLayer: Phaser.Tilemaps.TilemapLayer;
  constructor(
    public state: CarObject,
    private readonly scene: Phaser.Scene & {
      roadLayer: Phaser.Tilemaps.TilemapLayer;
      carGroup: Phaser.Physics.Arcade.Group;
    }
  ) {
    this.id = state.id;
    this.roadLayer = scene.roadLayer;
    this.maxSpeed = state.details.Car.speed;
    this.rotationSpeed = state.details.Car.rotation_speed;
    this.acceleration = 100;
    this.deceleration = 50;
    const carSprite = this.scene.physics.add.sprite(
      state.x,
      state.y,
      state.details.Car.skin
    );
    console.log(state);
    carSprite.setDepth(1);
    carSprite.setCollideWorldBounds(true);
    scene.carGroup.add(carSprite);
    this.sprite = carSprite;
    console.log("Car added.");
  }
  isActionable(userId: string): boolean {
    return true;
  }
  action(playerController: PlayerController) {
    if (!this.state.details) return;
    if (playerController.getControlledEntity() != this) {
      playerController.setControlledEntity(this);
      return;
    }
    console.log("Exiting control.");
    if (
      this.sprite.body?.velocity.x === 0 &&
      this.sprite.body.velocity.y === 0
    ) {
      playerController.resetToMain();
    }
  }
  takeControl() {
    if (!this.state.details.Car.driver_user_id) {
      console.log("Taking control as driver.");
    }
    console.log("Now taking control.");
  }
  removeControl() {
    console.log("Removing control.");
  }
  getSprite(): Phaser.Physics.Arcade.Sprite {
    return this.sprite;
  }
  private attemptReposition(direction: number): boolean {
    let impulse = 5;
    const maxImpulse = 30;
    while (impulse <= maxImpulse) {
      const testX = this.sprite.x - Math.cos(direction) * impulse;
      const testY = this.sprite.y - Math.sin(direction) * impulse;
      const testTile = this.roadLayer.getTileAtWorldXY(testX, testY);
      if (testTile) {
        this.sprite.x = testX;
        this.sprite.y = testY;
        return true;
      }
      impulse += 5;
    }
    return false;
  }
  updateInput(
    cursors: Phaser.Types.Input.Keyboard.CursorKeys,
    delta: number
  ): void {
    this.scene.cameras.main.startFollow(this.getSprite(), true, 0.08, 0.08);
    this.scene.cameras.main.setDeadzone(100, 100);
    if (this.state.details.Car.driver_user_id !== user.user?.sub) {
      console.log("Not the driver; no local control.");
      return;
    }
    if (this.isBumping) {
      return;
    }
    const dt = delta / 1000;
    const minRotationFactor = 0.2;
    const speedFactor = 1 - Math.min(this.currentSpeed / this.maxSpeed, 1);
    const rotationFactor =
      minRotationFactor + (1 - minRotationFactor) * speedFactor;
    if (this.currentSpeed > 0) {
      if (cursors.left.isDown) {
        this.sprite.rotation -= this.rotationSpeed * rotationFactor * dt;
      } else if (cursors.right.isDown) {
        this.sprite.rotation += this.rotationSpeed * rotationFactor * dt;
      }
    }
    const direction = this.sprite.rotation - Math.PI / 2;
    if (cursors.up.isDown) {
      this.currentSpeed += this.acceleration * dt;
      if (this.currentSpeed > this.maxSpeed) {
        this.currentSpeed = this.maxSpeed;
      }
    } else if (cursors.down.isDown) {
      this.currentSpeed -= this.breaking * dt;
      if (this.currentSpeed < 0) {
        this.currentSpeed = 0;
      }
    } else {
      this.currentSpeed = Math.max(
        0,
        this.currentSpeed - this.deceleration * dt
      );
    }
    const desiredVX = Math.cos(direction) * this.currentSpeed;
    const desiredVY = Math.sin(direction) * this.currentSpeed;
    const newX = this.sprite.x + desiredVX * dt;
    const newY = this.sprite.y + desiredVY * dt;
    const tile = this.roadLayer.getTileAtWorldXY(newX, newY);
    if (!this.sprite.body || !("setVelocity" in this.sprite.body)) {
      return;
    }
    if (tile) {
      this.sprite.body.setVelocity(desiredVX, desiredVY);
    } else {
      if (!this.isBumping) {
        this.isBumping = true;
        this.currentSpeed = 0;
        const repositioned = this.attemptReposition(direction);
        if (!repositioned) {
          this.sprite.x -= Math.cos(direction) * 5;
          this.sprite.y -= Math.sin(direction) * 5;
        }
        this.sprite.scene.tweens.add({
          targets: this.sprite,
          x: this.sprite.x,
          y: this.sprite.y,
          yoyo: true,
          duration: 50,
          ease: "Sine.easeInOut",
          onComplete: () => {
            this.isBumping = false;
          },
        });
        this.sprite.setTint(0xff0000);
        this.sprite.scene.time.delayedCall(100, () => {
          this.sprite.clearTint();
        });
      }
      this.sprite.body.setVelocity(0, 0);
    }
  }
  /**
   * Called when receiving updates from the server.
   */
  updateInputFromServer(state: CarObject, time: number, delta: number) {
    this.state = state;
    this.update(time, delta);
    this.lastServerUpdateTime = time;
  }
  /**
   * Update the car based on server state. Uses linear interpolation
   * to smoothly adjust position and rotation.
   */
  update(time: number, delta: number) {
    if (!this.state) {
      console.log("No state available yet.");
      return;
    }
    this.maxSpeed = this.state.details.Car.speed;
    this.rotationSpeed = this.state.details.Car.rotation_speed;
    this.sprite.setTexture(this.state.details.Car.skin);
    if (this.state.details.Car.driver_user_id === user.user?.sub) {
      return;
    }
    const elapsed = time - this.lastServerUpdateTime;
    const currentX = this.sprite.x;
    const currentY = this.sprite.y;
    const currentRotation = this.sprite.rotation;
    const targetX = this.state.x;
    const targetY = this.state.y;
    const targetRotation = this.state.rotation;
    const lerpFactor = Math.min(1, elapsed / this.expectedUpdateInterval);
    this.sprite.x = Phaser.Math.Linear(currentX, targetX, lerpFactor);
    this.sprite.y = Phaser.Math.Linear(currentY, targetY, lerpFactor);
    this.sprite.rotation = Phaser.Math.Angle.RotateTo(
      currentRotation,
      targetRotation,
      this.rotationSpeed * (delta / 1000)
    );
  }
  getInputState(): InputState {
    return {
      r: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
    };
  }
}
