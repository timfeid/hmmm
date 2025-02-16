import Phaser from "phaser";
import type { InputState, Actionable } from "./actionable";
import type { Controllable } from "./controllable";

export class PlayerController extends EventTarget {
  private controlledEntity: Controllable;

  constructor(
    private readonly userId: string,
    private readonly mainEntity: Controllable
  ) {
    super();
    this.controlledEntity = mainEntity;
  }

  resetToMain() {
    this.mainEntity.sprite.x = this.controlledEntity.sprite.x;
    this.mainEntity.sprite.y = this.controlledEntity.sprite.y;
    this.setControlledEntity(this.mainEntity);
  }

  action(actionables: Actionable[]) {
    const entitySprite = this.controlledEntity.sprite;
    const cx = entitySprite.x;
    const cy = entitySprite.y;
    const threshold = 32;

    for (const obj of actionables) {
      if (obj.isActionable(this.userId)) {
        const dist = Phaser.Math.Distance.Between(
          cx,
          cy,
          obj.sprite.x,
          obj.sprite.y
        );
        if (dist <= threshold) {
          obj.action(this);
          break;
        }
      }
    }
  }

  getSprite(): Phaser.GameObjects.GameObject {
    return this.controlledEntity.sprite;
  }

  getControlledEntity() {
    return this.controlledEntity;
  }

  setControlledEntity(entity: Controllable) {
    const old = this.controlledEntity;
    this.controlledEntity.removeControl();
    this.controlledEntity = entity;
    this.controlledEntity.takeControl();
    this.dispatchEvent(new CustomEvent("updated", { detail: { entity: old } }));

    return old;
  }

  update(cursors: Phaser.Types.Input.Keyboard.CursorKeys, delta: number) {
    this.controlledEntity.updateInput(cursors, delta);
  }

  getInputState(): InputState {
    return this.controlledEntity.getInputState();
  }
}
