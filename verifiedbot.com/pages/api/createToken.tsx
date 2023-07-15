import { AWSError } from "aws-sdk";
import { withIronSessionApiRoute } from "iron-session/next";
import { NextApiHandler } from "next";
import { ironOptions } from "../../lib/config";
import { docClient} from "../../lib/db";
import { encrypt } from "aes-gcm-siv-wasm";


function get_current_json_time_string() {
    return JSON.parse(JSON.stringify(new Date()));
}

const handler: NextApiHandler = withIronSessionApiRoute(async (req, res) => {
  const { token, uteid } = req.body;
  const authKey = req.headers.authorization;

  let eid: string = uteid.replace('@utexas.edu', '');

  if(authKey !=  process.env.QUALTRICS_AUTHENTICATION) {
    res.status(401).send("Unauthorized.");
  } else {
      let encrypted_uteid = btoa(encrypt(eid, process.env.UT_EID_AES_KEY));

        const tr = docClient.transactWrite({
          TransactItems: [
            {
              Update: {
                TableName: "qualtrics_tokens",
                Key: {
                  token,
                },
                UpdateExpression: "set created_at = :created_at, encrypted_eid = :encrypted_eid",
                ExpressionAttributeValues: {
                  ":created_at": get_current_json_time_string(),
                  ":encrypted_eid": encrypted_uteid
                },
              },
            }
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
          res.status(201).send("Created.");
        }).catch((e) => {
          console.log(e, cancellationReasons!);
          res.status(403).send("Qualtrics token already exists.");
        });
  }

}, ironOptions);

export default handler;
