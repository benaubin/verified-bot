import { useMemo } from "react";
import { VerifiedClaims } from "../lib/token";

export type ReadableClaims = Omit<VerifiedClaims, "encrypted_eid"> & {encrypted_eid: string};

export function readableClaims(c: VerifiedClaims): ReadableClaims {
  return {
    ...c,
    encrypted_eid: btoa(String.fromCharCode(...c.encrypted_eid))
  }
}

export default function ClaimsData({ claims }: { claims: ReadableClaims }) {
  return (
    <>
      <table>
        {Object.entries(claims).map(([k, v]) => {
          return (
            <tr key={k}>
              <th scope="row">{k}</th>
              <td>
                {v instanceof Array
                  ? v.join(";")
                  : v}
              </td>
            </tr>
          );
        })}
      </table>
    </>
  );
}
