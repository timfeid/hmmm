import Phaser from "phaser";
import type { Actionable, InputState } from "./actionable";
import type { PlayerController } from "./player-controller";
import type { Controllable } from "./controllable";
import type { ServerUpdatable } from "./updatable";
import type { PersonObject } from "./utils";
import { user } from "../../stores/access-token.svelte";

export class Person implements Controllable, ServerUpdatable {
  id: string;
  public sprite: Phaser.Physics.Arcade.Sprite;
  public speed: number = 80;
  public rotationSpeed = 100;
  private lastServerUpdateTime: number = 0;
  public inControl = true;
  private expectedUpdateInterval: number = 64;

  constructor(
    public state: PersonObject,
    private readonly scene: Phaser.Scene & {
      personGroup: Phaser.Physics.Arcade.Group;
    }
  ) {
    this.id = state.id;
    const sprite = this.scene.physics.add.sprite(
      state.x,
      state.y,
      state.details.Person.skin
    );
    console.log(state);
    // sprite.setDisplaySize(this.sprite.width, sprite.displayHeight); sprite.body.setSize(this.sprite.width, sprite.height);
    sprite.setDepth(1);
    sprite.setCollideWorldBounds(true);
    scene.personGroup.add(sprite);
    this.sprite = sprite;
    console.log("added.", state.details.Person.skin);
    this.sprite.anims.play("idle");
  }

  updateInputFromServer(state: PersonObject, time: number, delta: number) {
    if (state.details.Person.user_id === user.user?.sub) {
      return;
    }
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
    this.sprite.anims.play("idle");
    // this.sprite.setVisible(!this.state.hidden);

    // if (
    //   this.state.details.Person.user_id !== user.user?.sub &&
    //   this.state.animation &&
    //   this.sprite.anims.currentAnim?.key !== this.state.animation
    // ) {
    //   this.sprite.anims.play(this.state.animation);
    // }
  }

  getSprite(): Phaser.Physics.Arcade.Sprite {
    return this.sprite;
  }

  isActionable(uid: string) {
    console.log("no reason to action on someone else yet?");
    return false;
    // return uid === this.state.owner_id && !this.inControl;
  }

  action(playerController: PlayerController) {
    console.log("actioned on the person");
    // playerController.setControlledEntity(this);
  }

  takeControl() {
    this.sprite.setVisible(true);
    this.inControl = true;
  }

  removeControl() {
    this.sprite.setVisible(false);

    this.inControl = false;
  }

  updateInput(cursors: Phaser.Types.Input.Keyboard.CursorKeys, delta: number) {
    this.scene.cameras.main.startFollow(this.getSprite(), true, 0.08, 0.08);
    this.scene.cameras.main.setDeadzone(50, 50);
    this.scene.cameras.main.setZoom(1);
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
    if (vx !== 0 || vy !== 0) {
      if (this.sprite.anims.currentAnim?.key !== "walk") {
        this.sprite.anims.play("walk");
      }
    } else {
      if (this.sprite.anims.currentAnim?.key !== "idle") {
        this.sprite.anims.play("idle");
      }
    }
    if (this.sprite.body && "setVelocity" in this.sprite.body) {
      this.sprite.body.setVelocity(vx, vy);
    }
  }

  getInputState(): InputState {
    return {
      r: Math.round(this.sprite.rotation * 1000) / 1000,
      x: Math.round(this.sprite.x),
      y: Math.round(this.sprite.y),
    };
  }
}
