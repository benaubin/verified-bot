import { config } from "aws-sdk";

config.update({
  region: process.env.VB_AWS_REGION,
  credentials: {
    accessKeyId: process.env.VB_AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.VB_AWS_SECRET_ACCESS_KEY!,
  },
});

import { DynamoDB, SQS } from "aws-sdk";
import { VerifiedClaims } from "./token";

export interface User {
  discord_id: String;
  token_requested_at: number;
  claims: VerifiedClaims;
}

export const docClient = new DynamoDB.DocumentClient();

export const getUser = async (discord_id: string) => {
  const user = await docClient
    .get({ TableName: "users", Key: { discord_id } })
    .promise();
  return user.Item as any as User;
};


const SQS_QueueUrl =
  "https://sqs.us-east-1.amazonaws.com/402762806873/eid_verification_requests";

/**
 * Sends a UT EID to an S3 SQS message queue, so that the verification request will be performed
 * by the verification server.
 * 
 * @param eid The user's ut eid
 */
export const requestToken = async (eid: string) => {
  const sqs = new SQS();
  const _res = await sqs.sendMessage({
    QueueUrl: SQS_QueueUrl,
    MessageBody: JSON.stringify({eid})
  }).promise();
};
