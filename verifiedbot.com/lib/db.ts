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
  encrypted_eid: String;
  claims: string;
}

export interface GuildRoles {
  guild_id: String;
  affiliation_roles: {String: number};
  school_roles: {String: number};
  major_roles: {String: number}
}

export const docClient = new DynamoDB.DocumentClient();

export const getUser = async (discord_id: string) => {
  const user = await docClient
    .get({ TableName: "users", Key: { discord_id } })
    .promise();
  return user.Item as any as User;
};

export const getGuildRoles = async (guild_id: string) => {
  const guild_roles = await docClient
      .get( { TableName: "guilds", Key: { guild_id }})
      .promise();
  return guild_roles.Item as any as GuildRoles;
};

export const setGuildRolesAttribute = async (guild_id: string, attribute: string, data: {String: number}) => {
  // guard against other table changes
  if (!["affiliation_roles", "major_roles", "school_roles"].includes(attribute)) {
    return null;
  }
  let _ = await docClient.update({
    TableName: "guilds",
    Key: {
      guild_id
    },
    UpdateExpression: `set ${attribute} = :data`,
    ExpressionAttributeValues: {
      ":data": JSON.stringify(data),
    },
    ReturnValues: "ALL_NEW"
  }).promise() as any as GuildRoles;
}

const EID_SQS_URL =
  "https://sqs.us-east-1.amazonaws.com/402762806873/eid_verification_requests";
const DISCORD_ID_SQS_URL =
    "https://sqs.us-east-1.amazonaws.com/402762806873/on-verification-update";

/**
 * Sends a UT EID to an S3 SQS message queue, so that the verification request will be performed
 * by the verification server.
 * 
 * @param eid The user's ut eid
 */
export const requestToken = async (eid: string) => {
  const sqs = new SQS();
  const _res = await sqs.sendMessage({
    QueueUrl: EID_SQS_URL,
    MessageBody: JSON.stringify({eid})
  }).promise();
};

/**
 * Sends a Discord ID to an S3 SQS message queue, which will be picked up by the Discord Bot so it
 * may update a user's server nickname and roles in every participating server.
 *
 * @param discord_id User's Discord ID
 */
export const becameVerified = async (discord_id: string) => {
  const sqs = new SQS;
  const _res = await sqs.sendMessage({
    QueueUrl: DISCORD_ID_SQS_URL,
    MessageBody: JSON.stringify({discord_id})
  }).promise();
}