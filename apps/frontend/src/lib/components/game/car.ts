import Phaser from "phaser";
import type { Actionable, InputState } from "./actionable.js";
import type { PlayerController } from "./player-controller.js";
// import type { Controllable } from "./costate.infodriveable.js";
import type { ServerUpdatable } from "./updatable.js";
import type { CarObject } from "./utils.js";
import { user } from "../../stores/access-token.svelte.js";
import type { Controllable } from "./controllable.js";

export class Car implements Controllable, ServerUpdatable {
  id: string;
  public sprite: Phaser.Physics.Arcade.Sprite;
  public speed: number;
  public rotationSpeed: number;
  private targetState?: InputState;
  private isBumping: boolean = false; // flag to prevent continuous bumps
  private readonly roadLayer: Phaser.Tilemaps.TilemapLayer;
  lastServerUpdateTime: number = 0;
  private expectedUpdateInterval: number = 64;

  constructor(
    public state: CarObject,
    private readonly scene: Phaser.Scene & {
      roadLayer: Phaser.Tilemaps.TilemapLayer;
      carGroup: Phaser.Physics.Arcade.Group;
    }
  ) {
    this.id = state.id;
    this.roadLayer = scene.roadLayer;
    this.speed = state.info.Car.speed;
    this.rotationSpeed = state.info.Car.rotation_speed;

    const carSprite = this.scene.physics.add.sprite(
      state.x,
      state.y,
      state.info.Car.skin
    );
    console.log(state);
    // carSprite.setDisplaySize(26, 58);
    // carSprite.body.setSize(26, 58);
    carSprite.setDepth(1);
    carSprite.setCollideWorldBounds(true);
    scene.carGroup.add(carSprite);
    this.sprite = carSprite;
    console.log("added.");
  }

  isActionable(userId: string): boolean {
    return true;
  }

  action(playerController: PlayerController) {
    // playerController.getControlledEntity().removeControl();
    // this.sprite.addToDisplayList();
    if (!this.state.action) {
      return;
    }

    if (playerController.getControlledEntity() != this) {
      // this.scene
      playerController.setControlledEntity(this);
      return;
    }

    console.log("exiting.");
    if (
      this.sprite.body?.velocity.x === 0 &&
      this.sprite.body.velocity.y === 0
    ) {
      playerController.resetToMain();
    }
  }

  takeControl() {
    if (!this.state.info.Car.driver_user_id) {
      console.log("we need to take control");
    }
    console.log("we are here taking control");
  }

  removeControl() {
    console.log("we are here removing control");
  }

  getSprite(): Phaser.Physics.Arcade.Sprite {
    return this.sprite;
  }

  /**
   * Update the car's input based on keyboard cursors and delta time.
   * This version also checks the road layer to determine if the new position is valid.
   * If not, it applies a bump and flash effect.
   *
   * @param cursors - the keyboard input keys
   * @param delta - delta time in ms
   * @param roadLayer - the tilemap layer used for road collision checks
   */
  updateInput(
    cursors: Phaser.Types.Input.Keyboard.CursorKeys,
    delta: number
  ): void {
    this.scene.cameras.main.startFollow(this.getSprite(), true, 0.08, 0.08);
    this.scene.cameras.main.setDeadzone(100, 100);

    if (this.state.info.Car.driver_user_id !== user.user?.sub) {
      console.log("not the driver, sit back n relax");
      return;
    }
    const dt = delta / 1000;
    let desiredVX = 0;
    let desiredVY = 0;

    if (cursors.left.isDown) {
      this.sprite.rotation -= this.rotationSpeed * dt;
    } else if (cursors.right.isDown) {
      this.sprite.rotation += this.rotationSpeed * dt;
    }

    const direction = this.sprite.rotation - Math.PI / 2;

    if (cursors.up.isDown) {
      desiredVX = Math.cos(direction) * this.speed;
      desiredVY = Math.sin(direction) * this.speed;
    } else if (cursors.down.isDown) {
      desiredVX = -Math.cos(direction) * this.speed;
      desiredVY = -Math.sin(direction) * this.speed;
    }

    const newX = this.sprite.x + desiredVX * dt;
    const newY = this.sprite.y + desiredVY * dt;
    const tile = this.roadLayer.getTileAtWorldXY(newX, newY);

    if (!this.sprite.body || !("setVelocity" in this.sprite.body)) {
      return;
    }

    if (tile) {
      this.sprite.body.setVelocity(desiredVX, desiredVY);
    } else {
      this.sprite.body.setVelocity(0, 0);

      if (!this.isBumping) {
        this.isBumping = true;
        let bumpX = 0,
          bumpY = 0;
        if (Math.abs(desiredVX) >= Math.abs(desiredVY)) {
          bumpX = desiredVX >= 0 ? -5 : 5;
        } else {
          bumpY = desiredVY >= 0 ? -5 : 5;
        }
        this.sprite.scene.tweens.add({
          targets: this.sprite,
          x: this.sprite.x + bumpX,
          y: this.sprite.y + bumpY,
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
    }
  }

  updateInputFromServer(state: CarObject, time: number, delta: number) {
    this.state = state;
    this.update(time, delta);
    this.lastServerUpdateTime = time;
  }

  update(time: number, delta: number) {
    if (!this.state) {
      console.log("no state yet");
      return;
    }
    this.speed = this.state.info.Car.speed;
    this.rotationSpeed = this.state.info.Car.rotation_speed;
    // console.log("skin", this.state.info.Car.skin);
    this.sprite.setTexture(this.state.info.Car.skin);
    // this.sprite.anims.play(skin)
    if (this.state.info.Car.driver_user_id === user.user?.sub) {
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

    if (
      this.state.info.Car.driver_user_id !== user.user?.sub &&
      this.state.animation &&
      this.sprite.anims.currentAnim?.key !== this.state.animation
    ) {
      this.sprite.anims.play(this.state.animation);
    }
  }

  getInputState(): InputState {
    return {
      rotation: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
      hidden: !this.sprite.visible,
      animation: this.sprite.anims.currentAnim?.key,
    };
  }
}
