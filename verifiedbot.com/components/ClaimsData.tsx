import { useMemo } from "react";
import { VerifiedClaims } from "../lib/token";

export default function ClaimsData({claims}: {claims: VerifiedClaims}) {
  return (
    <>
      <table>
        {
          Object.entries(claims).map(([k, v]) => {
            return (
              <tr>
                <th scope="row">{k.replaceAll("_", " ")}</th>
                <td>
                  {k == "encrypted_eid"
                    ? btoa(String.fromCharCode(...claims.encrypted_eid))
                    : v instanceof Array
                    ? v.join(";")
                    : v}
                </td>
              </tr>
            );
          })
        }
      </table>
    </>
  );
}
