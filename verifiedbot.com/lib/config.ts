

import { IronSessionOptions } from "iron-session";

declare module "iron-session" {
  interface IronSessionData {
    oauthState?: string;
    csrfToken?: string;
    discordAuth?: {
      id: string;
      token: string;
      expiresAt: number;
    };
  }
}

export const ironOptions: IronSessionOptions = {
  cookieName: "verfiedbot_session",
  password: process.env.IRON_SESSION_PASSWORD!,
  cookieOptions: {
    secure: process.env.NODE_ENV === "production",
  },
};

export const DISCORD_API_BASE = "https://discord.com/api";
