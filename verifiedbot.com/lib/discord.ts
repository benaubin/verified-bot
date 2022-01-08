import { DISCORD_API_BASE } from "./config";

export interface DiscordUser {
  id: string;
  username: string;
  discriminator: string;
  avatar: string;
}

export const getDiscordUser = async (token: string) => {
  const res = await fetch(DISCORD_API_BASE + "/users/@me", {
    headers: new Headers({
      authorization: "Bearer " + token,
    }),
  });
  return (await res.json()) as DiscordUser;
}