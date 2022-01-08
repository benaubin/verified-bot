import crypto from "crypto";
import msgpack from "@msgpack/msgpack";

export interface VerifiedClaims {
  encrypted_eid: Buffer | Uint8Array,
  major: String[],
  school: String[],
  affiliation: String[]
}

export function decodeToken(token: String): VerifiedClaims | false {
  const buf = Buffer.from(token, "base64url");
  const data = buf.slice(0, buf.length - 32);
  const hash = buf.slice(buf.length - 32);

  const hmac = crypto.createHmac("sha256", process.env.SHARED_KEY!);
  hmac.update(data);
  const digest = hmac.digest();

  const valid = crypto.timingSafeEqual(hash, digest);
  if (!valid) return false;

  const [encrypted_eid, major, school, affiliation] = msgpack.decode(data) as [Buffer, String[], String[], String[]];

  return {
    encrypted_eid,
    major,
    school,
    affiliation
  }
}
