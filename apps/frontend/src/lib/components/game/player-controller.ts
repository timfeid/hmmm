import Phaser from "phaser";
import type { InputState, Actionable } from "./actionable";

export class PlayerController {
  private controlledEntity: Actionable;

  constructor(
    private readonly userId: string,
    initialEntity: Actionable
  ) {
    this.controlledEntity = initialEntity;
  }

  action(actionables: Actionable[]) {
    const entitySprite = this.controlledEntity.sprite;
    const cx = entitySprite.x;
    const cy = entitySprite.y;
    const threshold = 32;

    for (const obj of actionables) {
      if (obj.isActionable(this.userId) && this.controlledEntity !== obj) {
        const dist = Phaser.Math.Distance.Between(
          cx,
          cy,
          obj.sprite.x,
          obj.sprite.y
        );
        if (dist <= threshold) {
          // Switch control if necessary.
          this.setControlledEntity(obj);
          obj.action();
          break;
        }
      }
    }
  }

  getSprite(): Phaser.GameObjects.GameObject {
    return this.controlledEntity.sprite;
  }

  setControlledEntity(entity: Actionable) {
    this.controlledEntity = entity;
  }

  update(cursors: Phaser.Types.Input.Keyboard.CursorKeys, delta: number) {
    this.controlledEntity.updateInput(cursors, delta);
  }

  getInputState(): InputState {
    return this.controlledEntity.getInputState();
  }
}
