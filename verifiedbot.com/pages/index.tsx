import type { NextPage } from 'next'
import Head from 'next/head'
import Image from 'next/image'
import Link from 'next/link';
import { withIronSessionSsr } from "iron-session/next";


import bg from "../assets/marek-piwnicki-B19q-khdj1E-unsplash.jpg";
import { useEffect, useMemo, useState } from 'react';
import { ironOptions } from '../lib/config';
import { getDiscordOauthLink } from "../lib/discord"

interface Props {
  oauthLink: string;
}


export const getServerSideProps = withIronSessionSsr(async (ctx) => {
  const oauthLink = await getDiscordOauthLink(ctx);
  return {
    props: {oauthLink},
  };
}, ironOptions);

const Home = ({ oauthLink }: Props) => {
  useEffect(() => {
    if (window.location.hash == "#privacy") {
      const p = document.getElementById("privacy") as HTMLDetailsElement;
      p.open = true;
      p.scrollIntoView(false);
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
        <p>
          Verified Bot strengthens Discord communities by providing an optional
          mechanism for users to verify their student status.
        </p>

        <p>
          <a href={oauthLink}>Connect with Discord.</a>
        </p>

        <Image src={bg} placeholder="blur"></Image>
        <p>
          <h2>FAQs:</h2>
          <details id="privacy">
            <summary>What is the Privacy Policy?</summary>
            <p>
              Verified Bot seeks to protect your privacy by minimizing data
              collection as much as possible. When we do need to store your
              data, we use strong encryption to ensure your information stays
              secure.
            </p>
            <p>
              When you connect your Discord account, we gain access to your
              public Discord profile, but never your credentials. We use this
              access in order to prevent abuse and make sure you verify an
              account you own.
            </p>
            <p>
              When you submit a verification request, we use your student ID and
              the university directory to send an email with an encrypted
              version of your student ID. We don&apos;t store your student ID until
              you confirm verification.
            </p>
            <p>
              Once you confirm your ID, we store your encrypted student ID, as
              well as public information about your student status (such as your
              affiliation, major and school). You&apos;ll be able to see this
              information before you confirm your ID. This data is made
              available to anyone with your Discord ID, such as servers you are
              a part of.
            </p>
            <p>
              Your encrypted student ID can be used to uniquely identify you as
              a student of your university, but does not reveal your actual student
              ID except with knowledge of a secret key. We will only decrypt your
              ID if we are obligated to do so by law, or upon request of a
              university official.
            </p>
            <p>
              To be clear, this is a free service, which is provided as-is without a
              warranty of any kind. We don&apos;t guarantee that the service will operate
              as intended, and as a condition of using our service, you assume all
              risks. You can (and should) read our{" "}
              <a href="https://github.com/verified-bot/verified-bot">
                source code
              </a>{" "}
              for yourself.
            </p>
          </details>
          <details>
            <summary>Is this an official university service?</summary>
            <p>
              No. Verified Bot is run by UT students, and is not endorsed by or affiliated
              with the university.
            </p>
          </details>
          <details>
            <summary>Which universities is Verified Bot available at?</summary>
            <p>
              Verified Bot is currently available at The University of Texas at
              Austin, but{" "}
              <a href="https://github.com/arpan-dhatt/utexas-verify-discord-bot">
                contributions
              </a>{" "}
              are welcome!
            </p>
          </details>
        </p>
        <p>
          Made by <a href="https://benaubin.com">Ben Aubin</a>,{" "}
          <a href="https://arpan.one/">Arpan Dhatt</a>, and{" "}
          <a href="https://nickorlow.com/">Nicholas Orlowsky</a>.{" "}
        </p>
        <p>
          Open source on{" "}
          <a href="https://github.com/arpan-dhatt/utexas-verify-discord-bot">
            GitHub
          </a>
          .
        </p>
      </main>

      <style jsx>{`
        .title {
          text-align: center;

          margin: 50px 0;
        }
      `}</style>
    </div>
  );
};

export default Home
