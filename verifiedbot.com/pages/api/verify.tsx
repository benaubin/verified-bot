import { AWSError } from "aws-sdk";
import { withIronSessionApiRoute } from "iron-session/next";
import { NextApiHandler } from "next";
import { ironOptions } from "../../lib/config";
import { docClient } from "../../lib/db";
import { decodeToken } from "../../lib/token";
import {getDiscordGuildsForUser, getGuildMember, setMemberNick} from "../../lib/discord";

const handler: NextApiHandler = withIronSessionApiRoute(async (req, res) => {
  const { token, csrf_token } = req.body;
  const discord_id = req.session.discordAuth?.id;

  if (
    discord_id == null ||
    req.cookies.csrf_token == null ||
    req.cookies.csrf_token != csrf_token
  ) {
    res.status(401).send("Unauthorized");
    return;
  }

  const payload = decodeToken(token);
  if (!payload) {
    res.status(401).send("Bad token");
    return;
  }

  const { encrypted_eid: encrypted_eid_bytes, ...claims } = payload;
  const encrypted_eid = Buffer.from(encrypted_eid_bytes);

  const tr = docClient.transactWrite({
    TransactItems: [
      {
        Update: {
          TableName: "ut_eids",
          Key: {
            encrypted_eid,
          },
          UpdateExpression: "set discord_id = :discord_id",
          ConditionExpression:
            "attribute_not_exists(discord_id) OR discord_id = :discord_id",
          ExpressionAttributeValues: {
            ":discord_id": discord_id,
          },
        },
      },
      {
        Update: {
          TableName: "users",
          Key: {
            discord_id,
          },
          UpdateExpression:
            "set encrypted_eid = :encrypted_eid, claims = :claims",
          ConditionExpression: "attribute_not_exists(encrypted_eid)",
          ExpressionAttributeValues: { ":encrypted_eid": encrypted_eid, ":claims": JSON.stringify(claims) },
        },
      },
    ],
  });

  let cancellationReasons: AWSError[] = [];
  tr.on("extractError", (r) => {
    if (r.error) {
      cancellationReasons = JSON.parse(
        r.httpResponse.body.toString()
      ).CancellationReasons;
    }
  });

  await tr.promise().then(async () => {
    res.status(200).send(claims);
    let guilds = await getDiscordGuildsForUser(process.env.DISCORD_BOT_TOKEN!, "Bot");
    for (let partialGuild of guilds) {
      try {
        let guildMember = await getGuildMember(partialGuild.id, discord_id);
        await setMemberNick(partialGuild.id, discord_id, guildMember.nick || guildMember.user.username + " ✓");
      } catch (e) {
        console.log(partialGuild.name, e);
      }
    }
  }).catch((e) => {
    console.log(e, cancellationReasons!);
    res.status(403).send("UT EID or Discord account already verified.");
  });
  
}, ironOptions);

export default handler;
