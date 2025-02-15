import Phaser from "phaser";
import type { Actionable, InputState } from "./actionable.js";
import type { PlayerController } from "./player-controller.js";

export class Car implements Actionable {
  public id: string;
  public sprite: Phaser.Physics.Arcade.Sprite;
  public speed: number;
  public rotationSpeed: number;
  private targetState?: InputState;
  private isBumping: boolean = false; // flag to prevent continuous bumps
  private readonly roadLayer: Phaser.Tilemaps.TilemapLayer;
  carLight: Phaser.GameObjects.Light;

  constructor(
    id: string,
    private readonly scene: Phaser.Scene & {
      roadLayer: Phaser.Tilemaps.TilemapLayer;
      carGroup: Phaser.Physics.Arcade.Group;
    },
    sprite: Phaser.Physics.Arcade.Sprite,
    speed = 100,
    rotationSpeed = 3
  ) {
    scene.carGroup.add(sprite);

    this.roadLayer = scene.roadLayer;
    this.id = id;
    this.sprite = sprite;
    this.speed = speed;
    this.rotationSpeed = rotationSpeed;

    // Create the car sprite normally.
    // this.sprite = this.scene.physics.add.sprite(
    //   tileSize * 20,
    //   tileSize * 21,
    //   "car-north"
    // );
    // Create a light that follows the car.
    this.carLight = this.scene.lights.addLight(
      this.sprite.x,
      this.sprite.y,
      100,
      0xffffff,
      1
    );
    // this.carLight.setScrollFactor(0); // so it moves with the camera

    // In update(), update the light's position:
    // this.sprite.on("changedata", () => {
    //   this.carLight.x = this.sprite.x;
    //   this.carLight.y = this.sprite.y;
    //   console.log("hi");
    // });
    // Alternatively, in update():
  }

  isActionable(userId: string): boolean {
    return true;
  }

  action(playerController: PlayerController) {
    playerController.getSprite().removeFromDisplayList();
    this.sprite.addToDisplayList();
    playerController.setControlledEntity(this);
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

    // Calculate the new potential position.
    const newX = this.sprite.x + desiredVX * dt;
    const newY = this.sprite.y + desiredVY * dt;

    // Check the tile at the new position.
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
    this.carLight.x = this.sprite.x;
    this.carLight.y = this.sprite.y;
  }

  getInputState(): InputState {
    return {
      up: false, // You could add more logic here if desired.
      down: false,
      left: false,
      right: false,
      rotation: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
    };
  }
}
