import { FetchTransport, WebsocketTransport, createClient } from "@rspc/client";
import type { Procedures } from "@gangsta/rusty";
import { PUBLIC_API_URL } from "$env/static/public";
import { browser } from "$app/environment";
import { get } from "svelte/store";
import { decodeJwt } from "jose";
import { isPast } from "date-fns/isPast";
import { refreshAccessToken } from "./auth";
import { user } from "./stores/access-token.svelte";

const transport = new FetchTransport(PUBLIC_API_URL, async (input, init) => {
  const refreshing = input.toString().includes("refresh_token");
  try {
    if (user.accessToken && !refreshing) {
      const payload = decodeJwt(user.accessToken);
      if (browser && isPast(new Date(payload.exp! * 1000))) {
        // console.log('refreshing access token');
        await refreshAccessToken();
      }
    }
  } catch (e) {
    console.error(e);
  }

  return fetch(input, {
    ...init,
    headers: {
      authorization: user.accessToken,
    },
  });
});

const wtransport = browser
  ? new WebsocketTransport(PUBLIC_API_URL.replace("http", "ws") + "/ws")
  : transport;
console.log(wtransport);
export const websocketClient = createClient<Procedures>({
  transport: wtransport,
});

export const client = createClient<Procedures>({
  transport,
});
