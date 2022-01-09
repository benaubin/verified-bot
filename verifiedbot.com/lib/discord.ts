import { IronSession } from "iron-session";
import { GetServerSidePropsContext } from "next";
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
};

export interface DiscordPartialGuild {
  "id": string,
  "name": string,
  "icon": string,
  "owner": boolean,
  "permissions": string,
  "features": string[]
}

export const getDiscordGuilds = async (token: string) => {
  const res = await fetch(DISCORD_API_BASE + "/users/@me/guilds", {
    headers: new Headers({
      authorization: "Bearer " + token,
    }),
  });
  if (res.ok) {
    return (await res.json()) as DiscordPartialGuild[];
  } else {
    console.error("Error fetching guilds: ", await res.json());
    return [];
  }
};

export namespace PERMISSIONS {
  export const MANAGE_GUILD = 1 << 5;
  const ADD_REACTIONS = 1 << 6;
  const SEND_MESSAGES = 1 << 11;
  const CHANGE_NICKNAME = 1 << 26;
  const MANAGE_NICKNAMES = 1 << 27;
  const MANAGE_ROLES = 1 << 28;
  export const requested =
    ADD_REACTIONS |
    SEND_MESSAGES |
    CHANGE_NICKNAME |
    MANAGE_NICKNAMES |
    MANAGE_ROLES;
}

export const getDiscordOauthLink = async (ctx: GetServerSidePropsContext) => {
  const { session } = ctx.req;
  if (session.oauthState == null) {
    session.oauthState = (await import("crypto"))
      .randomBytes(16)
      .toString("base64url");
    await session.save();
  }
  const oauthLink =
    `https://discord.com/api/oauth2/authorize?response_type=code` +
    `&client_id=${encodeURIComponent(
      process.env.NEXT_PUBLIC_DISCORD_CLIENT_ID!
    )}` +
    `&redirect_uri=${encodeURIComponent(
      process.env.NEXT_PUBLIC_DISCORD_REDIRECT!
    )}` +
    `&state=${encodeURIComponent((session as any).oauthState)}` +
    `&scope=${encodeURIComponent(["identify", "guilds"].join(" "))}`;
  return oauthLink;
}

export const getDiscordAddBotLink = (guild_id?: string) => {
  return (
    `https://discord.com/api/oauth2/authorize?client_id=${encodeURIComponent(
      process.env.NEXT_PUBLIC_DISCORD_CLIENT_ID!
    )}` +
    `&scope=bot` +
    `&permissions=${PERMISSIONS.requested}` +
    (guild_id ? `&guild_id=${guild_id}` : "")
  );
};
