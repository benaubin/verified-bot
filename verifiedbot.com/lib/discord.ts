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

export interface DiscordGuildMember {
  nick: string;
  roles: string[];
  permissions?: string;
}

export const getGuildMember = async (
  guildId: string,
  userId: string
): Promise<DiscordGuildMember> => {
  const res = await fetch(
    DISCORD_API_BASE + "/guilds/" + encodeURIComponent(guildId) + "/members/" + encodeURIComponent(userId),
    {
      headers: new Headers({
        authorization: "Bearer " + process.env.DISCORD_BOT_TOKEN!,
      }),
    }
  );
  if (res.ok) {
    return (await res.json()) as DiscordGuildMember;
  } else {
    throw await res.json();
  }
};

export const setMemberNick = async(guildId: string, userId: string, new_nick: string) => {
  const res = await fetch(
      DISCORD_API_BASE + "/guilds/" + encodeURIComponent(guildId) + "/members/" + encodeURIComponent(userId),
      {
        method: "patch",
        body: JSON.stringify({
          nick: new_nick
        }),
        headers: new Headers({
          authorization: "Bearer " + process.env.DISCORD_BOT_TOKEN!,
        }),
      }
  );
  if (res.ok) {
    return (await res.json()) as DiscordGuildMember;
  } else {
    throw await res.json();
  }
}

export const getDiscordGuildBotMember = async (
  guildId: string
): Promise<DiscordGuildMember> => {
  return await getGuildMember(
    guildId,
    process.env.NEXT_PUBLIC_DISCORD_CLIENT_ID!
  );
};

export interface DiscordGuild {
  id: string;
  name: string;
  icon: string;
}

export interface DiscordGuildRole {
  "id": string,
  "name": string,
  "permissions": string,
  "position": number,
  "color": number,
  "hoist": boolean,
  "managed": boolean,
  "mentionable": boolean
}

export const getDiscordGuild = async (guildId: string): Promise<DiscordGuild> => {
  const res = await fetch(
    DISCORD_API_BASE +
      "/guilds/" +
      encodeURIComponent(guildId),
    {
      headers: new Headers({
        authorization: "Bearer " + process.env.DISCORD_BOT_TOKEN!,
      }),
    }
  );
  if (res.ok) {
    return (await res.json()) as DiscordGuild;
  } else {
    throw await res.json();
  }
}

export interface DiscordPartialGuild {
  "id": string,
  "name": string,
  "icon": string,
  "owner": boolean,
  "permissions": string,
  "features": string[]
}
export const getDiscordGuildsForUser = async (token: string) => {
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

export const getDiscordOauthLink = async (ctx: GetServerSidePropsContext, guild_id?: string) => {
  const { session } = ctx.req;
  if (session.oauthState == null) {
    session.oauthState = (await import("crypto"))
      .randomBytes(16)
      .toString("base64url");
    await session.save();
  }

  const opts = {
    client_id: process.env.NEXT_PUBLIC_DISCORD_CLIENT_ID!,
    redirect_uri: process.env.NEXT_PUBLIC_DISCORD_REDIRECT!,
    state: session.oauthState,
    scope: "identify guilds",
    response_type: "code"
  } as Record<string, string>;

  if (guild_id) {
    opts["scope"] += " bot applications.commands";
    opts["permissions"] = PERMISSIONS.requested.toString();
    opts["guild_id"] = guild_id;
  }

  const q = Object.entries(opts).map(([k, v]) => `${k}=${encodeURIComponent(v)}`).join("&");
  return "https://discord.com/api/oauth2/authorize?" + q;
}

