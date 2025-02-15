import { client } from "../../lib/client.js";

export async function POST(req) {
  const token = req.cookies.get("game-refresh_token");
  if (token) {
    const accessToken = await client.mutation([
      "authentication.refresh_token",
      token,
    ]);
    if (accessToken.refresh_token) {
      return new Response(accessToken.refresh_token, { status: 200 });
    }
  }

  return new Response(null, { status: 401 });
}
