import { withIronSessionApiRoute } from "iron-session/next";
import { NextApiHandler } from "next";
import { ironOptions } from "../../lib/config";
import { docClient, User, requestToken } from "../../lib/db";

const RATE_LIMIT = 90 * 1000;

const handler: NextApiHandler = withIronSessionApiRoute(async (req, res) => {
  const { eid, csrf_token } = req.body;
  const discord_id = req.session.discordAuth?.id;
  if (
    discord_id == null ||
    req.cookies.csrf_token == null ||
    req.cookies.csrf_token != csrf_token
  ) {
    res.status(401).send("Unauthorized");
    return;
  }

  try {
    const now = Date.now();
    await docClient.update({
      TableName: "users",
      Key: {
        discord_id,
      },
      UpdateExpression: "set token_requested_at = :now",
      ConditionExpression:
        "attribute_not_exists(token_requested_at) OR token_requested_at <= :must_be_before",
      ExpressionAttributeValues: {
        ":now": now,
        ":must_be_before": now - RATE_LIMIT,
      },
      ReturnValues: "ALL_NEW"
    }).promise() as any as User;

    await requestToken(eid);

    res.status(200).send("Verification sent.");
  } catch (e) {
    if (e instanceof Error) {
      if ((e as any)["code"] == "ConditionalCheckFailedException") {
        res
          .status(429)
          .send(
            "Try again in a few minutes, or email support@verifiedbot.com."
          );
        return;
      }
    }
    throw e;
  }
}, ironOptions);

export default handler;
