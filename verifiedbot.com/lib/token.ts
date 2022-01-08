import crypto from "crypto";
import {decode} from "@msgpack/msgpack";

export interface VerifiedClaims {
  encrypted_eid: Buffer | Uint8Array,
  major: String[],
  school: String[],
  affiliation: String[]
}

const key = Buffer.from(process.env.SHARED_KEY!, "base64url");

export function decodeToken(token: String): VerifiedClaims | false {
  const buf = Buffer.from(token, "base64url");
  const data = buf.slice(0, buf.length - 32);
  const inputHash = buf.slice(buf.length - 32);

  const hmac = crypto.createHmac("sha256", key);
  hmac.update(data);
  const validHash = hmac.digest();
  if (!crypto.timingSafeEqual(inputHash, validHash)) return false;

  const [encrypted_eid, major, school, affiliation] = decode(data) as [Buffer, String[], String[], String[]];
  return {
    encrypted_eid,
    major,
    school,
    affiliation
  }
}
