import Head from "next/head";
import { withIronSessionSsr } from "iron-session/next";
import { ironOptions } from "../lib/config";
import { getUser } from "../lib/db";
import {
  DiscordUser,
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
        destination: "/#",
      },
    };
  }

  const [discordUser, dbUser] = await Promise.all([
    getDiscordUser(discordAuth.token),
    getUser(discordAuth.id)
  ]);

  if (dbUser !== undefined) {
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
  token = token + "";
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



export default function Verify({ discordUser }: Props) {
  const [claims, setClaims] = useState<ReadableClaims | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const router = useRouter();
  let {qualtricsTokenId} = router.query;

  useEffect(() => {
    qualtricsTokenId = Array.isArray(qualtricsTokenId) ? qualtricsTokenId[0] : qualtricsTokenId;

    if (!qualtricsTokenId) {
      router.push("/app");
      return;
    }
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

        <p>Hi! You&apos;re about to verify your UT EID.</p>

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
            We will store your encrypted student ID, as well as public
            information about your student status, which you can see below.
            This data will be made available to anyone with your Discord ID,
            such as servers you are a part of.
          </li>
          <li>
            We may share your EID with university officials upon request, such
            as to investigate a harassment incident. Where permitted, we&apos;ll
            notify you if this occurs.
          </li>
          <li>
            This is a free service, provided as-is, and you assume all risks of
            using it. If you want to verify our privacy measures, read our {" "}
            <a target="_blank" href="https://github.com/verified-bot/verified-bot" rel="noreferrer">source code</a>.
          </li>
        </ol>

        <p>You&apos;re verifying this Discord account:</p>

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

        {message && <p className="error-text">{message}</p>}

        <p>
          <button
            disabled={loading}
            onClick={(e) => {
              qualtricsTokenId = Array.isArray(qualtricsTokenId) ? qualtricsTokenId[0] : qualtricsTokenId;
              setLoading(true);
              verify(qualtricsTokenId!)
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
          .error-text {
            color: rgb(255,59,48);
          }
        `}</style>
      </main>
    </div>
  );
}
