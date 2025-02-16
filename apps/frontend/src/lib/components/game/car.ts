import Phaser from "phaser";
import type { Actionable, InputState } from "./actionable.js";
import type { PlayerController } from "./player-controller.js";
import type { Controllable } from "./controllable.js";
import type { VisibleObject } from "@gangsta/rusty";
import type { ServerUpdatable } from "./updatable.js";

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
    public state: VisibleObject,
    private readonly scene: Phaser.Scene & {
      roadLayer: Phaser.Tilemaps.TilemapLayer;
      carGroup: Phaser.Physics.Arcade.Group;
    },
    sprite: Phaser.Physics.Arcade.Sprite,
    speed = 100,
    rotationSpeed = 3
  ) {
    scene.carGroup.add(sprite);
    this.id = state.id;
    this.roadLayer = scene.roadLayer;
    this.sprite = sprite;
    this.speed = speed;
    this.rotationSpeed = rotationSpeed;
  }

  isActionable(userId: string): boolean {
    return true;
  }

  action(playerController: PlayerController) {
    // playerController.removeControl();
    // this.sprite.addToDisplayList();
    if (playerController.getControlledEntity() != this) {
      playerController.setControlledEntity(this);
      return;
    }

    if (
      this.sprite.body?.velocity.x === 0 &&
      this.sprite.body.velocity.y === 0
    ) {
      playerController.resetToMain();
    }
  }

  takeControl() {
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

    const dt = delta / 1000;
    let desiredVX = 0;
    let desiredVY = 0;

    // Rotation control using left/right keys.
    if (cursors.left.isDown) {
      this.sprite.rotation -= this.rotationSpeed * dt;
    } else if (cursors.right.isDown) {
      this.sprite.rotation += this.rotationSpeed * dt;
    }

    // Calculate the intended movement direction.
    // Our car image is drawn so that 0 rotation faces upward.
    // We subtract π/2 to have the car move in the direction it's facing.
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

    const newX = this.sprite.x + desiredVX * dt;
    const newY = this.sprite.y + desiredVY * dt;
    const tile = this.roadLayer.getTileAtWorldXY(newX, newY);

    if (tile) {
      // If valid (on a road), allow movement.
      this.sprite.body.setVelocity(desiredVX, desiredVY);
    } else {
      // Not a valid road tile: stop movement.
      this.sprite.body.setVelocity(0, 0);

      // Trigger the bump effect only if not already bumping.
      if (!this.isBumping) {
        this.isBumping = true;
        // Determine a small offset based on movement direction.
        let bumpX = 0,
          bumpY = 0;
        // Use the dominant direction (or a fixed offset if desired).
        if (Math.abs(desiredVX) >= Math.abs(desiredVY)) {
          bumpX = desiredVX >= 0 ? -5 : 5;
        } else {
          bumpY = desiredVY >= 0 ? -5 : 5;
        }
        // Apply a small tween to simulate a bump.
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
        // Flash red briefly.
        this.sprite.setTint(0xff0000);
        this.sprite.scene.time.delayedCall(100, () => {
          this.sprite.clearTint();
        });
      }
    }
  }

  updateInputFromServer(state: VisibleObject, time: number, delta: number) {
    this.state = state;
    this.update(time, delta);
    this.lastServerUpdateTime = time;
  }

  update(time: number, delta: number) {
    if (!this.state) {
      console.log("no state yet");
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
      rotation: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
      hidden: !this.sprite.visible,
    };
  }
}
