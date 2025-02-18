import type {
  CarDetails,
  GameObject,
  PersonDetails,
  PersonSkin,
} from "@gangsta/rusty";

export type CarObject = GameObject & {
  info: {
    Car: CarDetails;
  };
};

export function isCar(data: GameObject): data is CarObject {
  return "Car" in data.info;
}

export type PersonObject = GameObject & {
  info: {
    Person: PersonDetails;
  };
};

export function isPerson(data: GameObject): data is PersonObject {
  return "Person" in data.info;
}
