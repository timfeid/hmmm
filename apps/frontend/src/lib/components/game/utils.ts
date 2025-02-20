import type {
  CarDetails,
  OutgoingGameObject,
  PersonDetails,
  PersonSkin,
} from "@gangsta/rusty";

export type CarObject = OutgoingGameObject & {
  details: {
    Car: CarDetails;
  };
};

export function isCar(data: OutgoingGameObject): data is CarObject {
  return "Car" in data.details;
}

export type PersonObject = OutgoingGameObject & {
  details: {
    Person: PersonDetails;
  };
};

export function isPerson(data: OutgoingGameObject): data is PersonObject {
  return "Person" in data.details;
}
