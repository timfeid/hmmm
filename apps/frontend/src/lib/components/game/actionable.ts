import type { PlayerController } from "./player-controller";

export type InputState = {
  rotation: number;
  x: number;
  y: number;
  hidden: boolean;
  animation?: string;
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
