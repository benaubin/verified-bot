import Head from "next/head";
import { GetServerSidePropsContext, GetStaticPropsResult } from "next";
import {
  getDiscordGuild,
  DiscordGuildMember,
  PERMISSIONS,
  getDiscordGuildBotMember,
  getGuildMember,
  getDiscordOauthLink,
} from "../../../lib/discord";
import { withIronSessionSsr } from "iron-session/next";
import { ironOptions } from "../../../lib/config";

interface Props {
  guildMember: DiscordGuildMember
}

export const getServerSideProps = withIronSessionSsr(async (ctx) => {
  const { guild_id } = ctx.params as {guild_id: string};
  const { discordAuth } = ctx.req.session;
  if (discordAuth == null || discordAuth.expiresAt < Date.now() / 1000 + 60) {
    return {
      redirect: {
        statusCode: 303,
        destination: "/",
      },
    };
  }

  let botMember;
  try {
    botMember = await getDiscordGuildBotMember(guild_id);
  } catch (e) {
    console.warn("bot member missing", e);
    return {
      redirect: {
        permanent: false,
        destination: await getDiscordOauthLink(ctx, guild_id),
      },
    };
  }
  const botPermissions = Number.parseInt(botMember.permissions || "0");
  if ((botPermissions & PERMISSIONS.requested) !== PERMISSIONS.requested) {
    return {
      redirect: {
        permanent: false,
        destination: await getDiscordOauthLink(ctx, guild_id),
      },
    };
  }

  let guildMember: DiscordGuildMember;
  try {
    guildMember = await getGuildMember(guild_id, discordAuth.id);
  } catch (e) {
    console.warn("fetching guild member", e);
    return {
      redirect: {
        permanent: false,
        destination: await getDiscordOauthLink(ctx, guild_id),
      },
    };
  }

  // const memberPermissions = Number.parseInt(guildMember.permissions || "0");

  return {
    props: {guildMember: guildMember}
  }
}, ironOptions);

export default function Guild({guildMember}: Props) {
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

        <h2>{guildMember}</h2>

        
        <style jsx>{`
        `}</style>
      </main>
    </div>
  );
}
