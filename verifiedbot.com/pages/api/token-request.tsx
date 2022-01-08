import { config, DynamoDB } from "aws-sdk";
import { withIronSessionApiRoute } from "iron-session/next/dist";
import { NextApiHandler } from "next";
import { ironOptions } from "../../lib/config";
import { docClient, getUser, User } from "../../lib/db";

const RATE_LIMIT = 30 * 1000;

const handler: NextApiHandler = withIronSessionApiRoute(async (req, res) => {
  const { eid, csrf_token } = req.body;
  const discord_id = req.session.discordAuth?.id;
  if (
    discord_id == null ||
    req.session.csrfToken == null ||
    req.session.csrfToken != csrf_token
  ) {
    res.status(401).send("unauthorized");
    return;
  }

  const now = Date.now();
  const user: User = await docClient.update({
    TableName: "users",
    Key: {
      discord_id,
    },
    UpdateExpression: "set token_requested_at = :now",
    ConditionExpression:
      "attribute_not_exists(token_requested_at) OR token_requested_at <= :must_be_before",
    ExpressionAttributeValues: {
      now,
      must_be_before: now - RATE_LIMIT,
    },
    ReturnValues: "ALL_NEW"
  }).promise() as any as User;

  if (user.token_requested_at != now) {
    res.status(429).send("Try again in a few seconds.");
  }

  console.log(eid);

  res.status(204);
}, ironOptions);

export default handler;
