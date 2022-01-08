import Head from "next/head";
import { Suspense, useMemo, useState } from "react";
import { withIronSessionSsr } from "iron-session/next";
import { DISCORD_API_BASE, ironOptions } from "../lib/config";
import { getUser, User } from "../lib/db";
import {
  DiscordPartialGuild,
  DiscordUser,
  getDiscordAddBotLink,
  getDiscordGuilds,
  getDiscordOauthLink,
  getDiscordUser,
  PERMISSIONS,
} from "../lib/discord";
import Link from "next/link";

interface Props {
  discordUser: DiscordUser;
  claims: User["claims"];
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
    getDiscordGuilds(discordAuth.token),
  ]);

  return {
    props: { discordUser, guilds, info: user?.claims || null },
  };
}, ironOptions);

const UtEIDForm = () => {
  const [eid, setEid] = useState("");
  return (
    <div>
      <p>Enter your UT EID to verify your Discord account.</p>
      <form>
        <input
          type="text"
          value={eid}
          onChange={(e) => {
            setEid(e.target.value);
          }}
        />
      </form>

      <p>
        <Link href="/#privacy">Privacy policy</Link>
      </p>
    </div>
  );
};

export default function App({ discordUser, claims, guilds }: Props) {
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
          {claims ? (
            <div>You have already verified your EID.</div>
          ) : (
            <UtEIDForm />
          )}
        </div>

        {
          managedGuilds.length > 0
          ? <div>
              <h2>Servers you manage:</h2>

              <p>
                You can add Verified Bot to any server you manage.
              </p>

              <ul>
                {managedGuilds.map((guild) => (
                  <li>
                    {guild.name} <a href={getDiscordAddBotLink(guild.id)} target="_top">Add</a>
                  </li>
                ))}
              </ul>
            </div>
          : <></>
        }

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
