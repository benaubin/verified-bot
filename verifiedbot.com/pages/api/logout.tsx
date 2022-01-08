import { withIronSessionApiRoute } from "iron-session/next";
import { ironOptions } from "../../lib/config";

export default withIronSessionApiRoute(async (req, res) => {
  await req.session.destroy();
  res.redirect("/");
}, ironOptions);
