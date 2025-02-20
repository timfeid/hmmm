import Phaser from "phaser";
import type { InputState, Actionable } from "./actionable";
import type { Controllable } from "./controllable";
import type { OutgoingGameObject } from "@gangsta/rusty";

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

  action(actionables: OutgoingGameObject[]) {
    const entitySprite = this.controlledEntity.sprite;
    const cx = entitySprite.x;
    const cy = entitySprite.y;

    for (const obj of actionables) {
      if (obj.action) {
        const threshold = obj.action.trigger_type.ActionKeyPressed;
        const dist = Phaser.Math.Distance.Between(
          cx,
          cy,
          entitySprite.x,
          entitySprite.y
        );
        if (dist <= threshold) {
          return obj;
        }
      }
    }

    return undefined;
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
