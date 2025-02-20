import type { PlayerController } from "./player-controller";

export type InputState = {
  r: number;
  x: number;
  y: number;
};

export interface Actionable {
  id: string;
  sprite: Phaser.GameObjects.Sprite;
  isActionable(userId: string): boolean;
  action(playerController: PlayerController): void;
  updateInput(
    cursors: Phaser.Types.Input.Keyboard.CursorKeys,
    delta: number
  ): void;
  getInputState(): InputState;
}
