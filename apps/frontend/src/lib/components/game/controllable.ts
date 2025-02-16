import type { Actionable } from "./actionable";

export interface Controllable extends Actionable {
  takeControl(): void;
  removeControl(): void;
}
