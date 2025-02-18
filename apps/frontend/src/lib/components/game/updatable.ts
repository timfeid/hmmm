import type { GameObject } from "@gangsta/rusty";

export interface ServerUpdatable {
  state: GameObject;
  updateInputFromServer(state: GameObject, time: number, delta: number): void;
}
