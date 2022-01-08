import { withIronSessionApiRoute } from "iron-session/next";
import { ironOptions } from "../../../lib/config";
import { getDiscordUser } from "../../../lib/discord";

export default withIronSessionApiRoute(async (req, res) => {
  const { state, code } = req.query;
  const { oauthState } = req.session as any;
  if (
    typeof code != "string" ||
    state == null ||
    oauthState == null ||
    oauthState !== state
  ) {
    res.redirect("/");
    return;
  }

  const data = new URLSearchParams();
  data.append("redirect_uri", process.env.DISCORD_REDIRECT!);
  data.append("client_id", process.env.DISCORD_CLIENT_ID!);
  data.append("client_secret", process.env.DISCORD_SECRET!);
  data.append("grant_type", "authorization_code");
  data.append("code", code);

  const tokenRes = await fetch("https://discord.com/api/oauth2/token", {
    method: "POST",
    body: data
  });
  
  const discordAuth = await tokenRes.json();
  const discordUser = await getDiscordUser(discordAuth.access_token);

  req.session.discordAuth = {
    id: discordUser.id,
    token: discordAuth.access_token,
    expiresAt: discordAuth.expires_in + Math.floor(Date.now()/1000)
  };
  await req.session.save();

  res.redirect("/app");
}, ironOptions);
