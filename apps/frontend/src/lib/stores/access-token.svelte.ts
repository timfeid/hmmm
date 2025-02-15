import { decodeJwt } from "jose";

export class UserDetails {
  accessToken = $state<string | undefined>();

  user = $derived.by(() => {
    if (!this.accessToken) {
      return undefined;
    }

    return decodeJwt<{ sub: string }>(this.accessToken);
  });
}

export const user = new UserDetails();
