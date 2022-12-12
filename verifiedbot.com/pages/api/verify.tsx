import { AWSError } from "aws-sdk";
import { withIronSessionApiRoute } from "iron-session/next";
import { NextApiHandler } from "next";
import { ironOptions } from "../../lib/config";
import {becameVerified, docClient} from "../../lib/db";
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

  const tr = docClient.transactWrite({
    TransactItems: [
      {
          Delete: {
            TableName: "qualtrics_tokens",
            Key: {
                token,
            },
            ConditionExpression: "valid = :valid",
            ExpressionAttributeValues: {
              ":valid": true,
            },
          },
      },
      {
        Update: {
          TableName: "users",
          Key: {
            discord_id
          },
          UpdateExpression: "set verified = :verified",
          ConditionExpression : "attribute_not_exists(discord_id)",
          ExpressionAttributeValues: { ":verified": true },
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
    await becameVerified(discord_id);
    res.status(200).send("Created.");
  }).catch((e) => {
    console.log(e, cancellationReasons!);
    if(cancellationReasons !== undefined) {
        if(cancellationReasons.length > 1 && cancellationReasons[1].Code == "ConditionalCheckFailed") {
            if(cancellationReasons![0].Code == "ConditionalCheckFailed") {
                res.status(403).send("Discord account already verified and bad UT EID.");
            } else {
                res.status(403).send("Discord account already verified.");
            }
        } else if(cancellationReasons.length > 0 && cancellationReasons[0].Code == "ConditionalCheckFailed") {
            res.status(403).send("Bad UT EID. Try signing in again.");
        }
    }
    res.status(403).send("Server error.");
  });

}, ironOptions);

export default handler;
