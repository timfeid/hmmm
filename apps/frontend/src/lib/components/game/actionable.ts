export type InputState = {
  up: boolean;
  down: boolean;
  left: boolean;
  right: boolean;
  rotation: number;
  x: number;
  y: number;
};

export interface Actionable {
  sprite: Phaser.GameObjects.Sprite;
  isActionable(userId: string): boolean;
  action(): void;
  updateInput(
    cursors: Phaser.Types.Input.Keyboard.CursorKeys,
    delta: number
  ): void;
  getInputState(): InputState;
}
