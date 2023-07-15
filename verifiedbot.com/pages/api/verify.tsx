import {AWSError} from "aws-sdk";
import {withIronSessionApiRoute} from "iron-session/next";
import {NextApiHandler} from "next";
import {ironOptions} from "../../lib/config";
import {becameVerified, docClient} from "../../lib/db";

const handler: NextApiHandler = withIronSessionApiRoute(async (req, res) => {
    const {token, csrf_token} = req.body;
    const discord_id = req.session.discordAuth?.id;

    if (
        discord_id == null ||
        req.cookies.csrf_token == null ||
        req.cookies.csrf_token != csrf_token
    ) {
        res.status(401).send("Unauthorized");
        return;
    }

    docClient.get(
        {
            TableName: "qualtrics_tokens",
            Key: {
                token,
            }
        },
        (err, data) => {
            if (err || data === undefined || data === null || data === {}) {
                res.status(403).send("Bad UT EID. Try signing in again.");
                return;
            }

            let encrypted_eid = data.Item.encrypted_eid;
            let expiry = new Date(data.Item.created_at);

            if ((new Date() - expiry) > (5 * 60 * 1000)) {
                res.status(403).send("Expired request. Try signing in again.");
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
                        },
                    },
                    {
                        Update: {
                            TableName: "users",
                            Key: {
                                discord_id
                            },
                            UpdateExpression: "set encrypted_eid = :encrypted_eid, claims = :claims",
                            ConditionExpression: "attribute_not_exists(discord_id) AND attribute_not_exists(encrypted_eid)",
                            ExpressionAttributeValues: {":encrypted_eid": encrypted_eid, ":claims": JSON.stringify({})},
                            // stringify empty claims for backwards compatibility
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

            tr.promise().then(async () => {
                await becameVerified(discord_id);
                res.status(200).send("Created.");
            }).catch((e) => {
                console.log(e, cancellationReasons!);
                if (cancellationReasons !== undefined) {
                    if (cancellationReasons.length > 1 && cancellationReasons[1].Code == "ConditionalCheckFailed") {
                        if (cancellationReasons![0].Code == "ConditionalCheckFailed") {
                            res.status(403).send("Discord account already verified and bad UT EID.");
                        } else {
                            res.status(403).send("Discord account already verified.");
                        }
                    } else if (cancellationReasons.length > 0 && cancellationReasons[0].Code == "ConditionalCheckFailed") {
                        res.status(403).send("Bad UT EID. Try signing in again.");
                    }
                }
                res.status(403).send("Server error.");
            });
        });
}, ironOptions);

export default handler;
