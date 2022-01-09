import Head from "next/head";
import { withIronSessionSsr } from "iron-session/next";
import { ironOptions } from "../lib/config";
import { getUser } from "../lib/db";
import {
  DiscordUser,
  getDiscordAddBotLink,
  getDiscordUser,
  PERMISSIONS,
} from "../lib/discord";
import Link from "next/link";
import { useLayoutEffect, useState } from "react";
import {decode} from "@msgpack/msgpack";
import {VerifiedClaims} from "../lib/token";
import ClaimsData, { readableClaims, ReadableClaims } from "../components/ClaimsData";
import { getCsrfToken } from "../lib/csrf";
import { useRouter } from "next/router";
import { useRef } from "react";
import { useEffect } from "react";

interface Props {
  discordUser: DiscordUser;
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

  const [discordUser, dbUser] = await Promise.all([
    getDiscordUser(discordAuth.token),
    getUser(discordAuth.id)
  ]);

  if (dbUser?.claims) {
    return {
      redirect: {
        statusCode: 303,
        destination: "/app#",
      },
    };
  }

  return {
    props: { discordUser },
  };
}, ironOptions);

const verify = async (token: string) => {
  const csrf_token = getCsrfToken();
  return fetch("/api/verify", {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      csrf_token,
      token,
    }),
  });
}

const token = typeof window === "undefined" ? null : window.location.hash.slice(1);
if (token) {
  window.history.replaceState({}, document.title, location.pathname);
}

export default function Verify({ discordUser }: Props) {
  const [claims, setClaims] = useState<ReadableClaims | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const router = useRouter();

  useEffect(() => {
    if (!token) {
      router.push("/app");
      return;
    }

    console.log(token);

    const byteStr = atob(token.replaceAll("-", "+").replaceAll("_", "/"));
    const buf = new Uint8Array(byteStr.length - 32);
    for (let i = 0; i < buf.length; i++) buf[i] = byteStr.charCodeAt(i);
    const [encrypted_eid, major, school, affiliation] = decode(buf) as [Buffer, String[], String[], String[]];
    setClaims(readableClaims({ encrypted_eid, major, school, affiliation }));
  }, []);

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

        <p>Hi! You're about to verify your UT EID.</p>

        <p>Before you do that, you should know a few things:</p>

        <ol>
          <li>
            Verified Bot is student-run, and is not an official service of UT or
            Discord.
          </li>
          <li>
            Verifying your UT EID permanently associates your Discord account
            with your UT EID, and applies to every server.
          </li>
          <li>
            We will share your affiliation, department and major with servers
            you are a part of, and may update this based on public directory
            information, until you ask us to stop.
          </li>
          <li>
            We store an encrypted version of your EID, which requires a secret
            key to decypher. We will only share your unencrypted EID if we are
            obligated to do so by law, or upon request of a UT official, such
            as to investigate a harassment incident.
          </li>
          <li>We will not store any other information about you.</li>
        </ol>

        <p>You're verifying this Discord account:</p>

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
              Not you? <Link href="/api/logout">Change user</Link>
            </div>
          </div>
        </div>

        <p>
          This is the verified information which will be publicly associated
          with your Discord account:
        </p>

        {claims && <ClaimsData claims={claims}></ClaimsData>}

        {message && <p>{message}</p>}

        <p>
          <button
            disabled={loading}
            onClick={(e) => {
              setLoading(true);
              verify(token!)
                .then(async (res) => {
                  if (res.ok) {
                    router.push("/app#");
                  } else {
                    const message = await res.text();
                    setMessage(message);
                  }
                })
                .finally(() => {
                  setLoading(false);
                });
              e.preventDefault();
            }}>
            Verify my Discord account
          </button>
        </p>

        <p>
          If you have questions, email{" "}
          <a href="mailto:support@verifiedbot.com">support@verifiedbot.com</a>.
        </p>

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