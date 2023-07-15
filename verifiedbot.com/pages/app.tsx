import Head from "next/head";
import { useState } from "react";
import { withIronSessionSsr } from "iron-session/next";
import { ironOptions } from "../lib/config";
import { getUser } from "../lib/db";
import {
  DiscordPartialGuild,
  DiscordUser,
  getDiscordGuildsForUser,
  getDiscordUser,
  PERMISSIONS,
} from "../lib/discord";
import Link from "next/link";
import { getCsrfToken } from "../lib/csrf";

interface Props {
  discordUser: DiscordUser;
  isVerified: boolean;
  guilds: DiscordPartialGuild[];
}

export const getServerSideProps = withIronSessionSsr(async (ctx) => {
  const { discordAuth } = ctx.req.session;
  if (discordAuth == null || discordAuth.expiresAt < Date.now() / 1000 + 60) {
    return {
      redirect: {
        statusCode: 303,
        destination: "/",
      },
    };
  }

  const [discordUser, user, guilds] = await Promise.all([
    getDiscordUser(discordAuth.token),
    getUser(discordAuth.id),
    getDiscordGuildsForUser(discordAuth.token),
  ]);

  const isVerified = user !== undefined && user !== null;
  return {
    props: { discordUser, guilds,  isVerified},
  };
}, ironOptions);

const VerificationForm = () => {
  return (
    <div>
        <p>Click the button below to verify your Discord account.</p>
        <input
            type="submit"
            onClick={()=>{window.location.href=process.env.NEXT_PUBLIC_QUALTRICS_URI}}
            value="Begin Verification"/>
        <p>
            <Link href="/#privacy">Privacy policy</Link>
        </p>
    </div>
  );
};

export default function App({ discordUser, isVerified, guilds }: Props) {
  const managedGuilds = guilds.filter(
    (guild) =>
      (Number.parseInt(guild.permissions) & PERMISSIONS.MANAGE_GUILD) != 0
  );

  return (
    <div className="container">
      <Head>
        <title>Verified Bot</title>
        <meta
          name="description"
          content="Discord Bot for verifing student status."
        />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main>
        <h1>Verified Bot</h1>

        <div className="discord-info">
          <img
            className="avatar"
            src={`https://cdn.discordapp.com/avatars/${encodeURIComponent(
              discordUser.id
            )}/${encodeURIComponent(discordUser.avatar)}.png`}></img>
          <div>
            <div className="name">{discordUser.username}</div>
            <div className="discriminator">#{discordUser.discriminator}</div>
            <div>
              Not you? <Link href="/api/logout">Logout</Link>
            </div>
          </div>
        </div>

        <div>
          {isVerified ? (
            <>
              <h2>You are verified!</h2>
            </>
          ) : (
            <VerificationForm />
          )}
        </div>

        {managedGuilds.length > 0 ? (
          <div>
            <h2>Servers you manage:</h2>

            <p>You can add Verified Bot to any server you manage.</p>

            <ul>
              {managedGuilds.map((guild) => (
                <li key={guild.id}>
                  <Link href={`/guild/[guild_id]`} as={`/guild/${guild.id}`}>
                    {guild.name}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ) : (
          <></>
        )}

        <style jsx>{`
          .discord-info {
            display: flex;
            align-items: center;
          }
          .avatar {
            border-radius: 100%;
            height: 64px;
            width: 64px;
            margin: 12px;
          }
          .name {
            font-weight: 700;
          }
          .discriminator {
            font-weight: 300;
          }
        `}</style>
      </main>
    </div>
  );
}
