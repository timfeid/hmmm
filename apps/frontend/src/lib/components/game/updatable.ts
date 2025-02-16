import type { VisibleObject } from "@gangsta/rusty";

export interface ServerUpdatable {
  state: VisibleObject;
  updateInputFromServer(
    state: VisibleObject,
    time: number,
    delta: number
  ): void;
}
