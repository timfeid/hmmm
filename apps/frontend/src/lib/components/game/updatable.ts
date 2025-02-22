import type { OutgoingGameObject } from "@gangsta/rusty";

export interface ServerUpdatable {
  state: OutgoingGameObject;
  updateInputFromServer(
    state: OutgoingGameObject,
    time: number,
    delta: number
  ): void;
}
