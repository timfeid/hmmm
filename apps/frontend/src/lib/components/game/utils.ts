import type {
  CarSkin,
  PersonSkin,
  VisibleObject,
  VisibleObjectType,
} from "@gangsta/rusty";

export type CarObject = VisibleObject & {
  type: {
    Car: [CarSkin, number, number];
  };
};

export function isCar(data: VisibleObject): data is CarObject {
  return "Car" in data.type;
}

export type PersonObject = VisibleObject & {
  type: {
    Person: PersonSkin;
  };
};

export function isPerson(data: VisibleObject): data is PersonObject {
  return "Person" in data.type;
}
