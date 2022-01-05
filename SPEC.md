## API Specification

Describes the API specification of the UT affiliation verification service.

## POST /request-verification.cgi

Begin a verification request. The service uid must have been verified.

Body is a signed JWT, using a pre-shared secret, of the following form:

```
{
    ut_eid: "bha366",
    service: "discord",
    service_id: "[discord id]",
    redirect: "[url for where to redirect the user after succesful login]"
}
```

If the user is not already verified, we send an email using the eid (`bha366@eid.utexas.edu`), with a new jwt token, see below. 

The pre-shared secret authorizes the service to begin verification requests, and prevents unauthorized use of our verification service.

We may need to consider limiting this endpoint, to prevent spamming/dos attacks. It should be fine for now, though.

### GET /verify.cgi#token

The link sent to the user via their eid.utexas.edu email.

The token consists of the following:

```
{
    ut_eid: "bha366",
    service: "discord",
    service_id: "[discord id]",
    exp: "[15 minutes]",
    redirect: "[url for where to redirect the user after succesful login]"
}
```

It is signed with a seperate secret than the one used for the discord bot.

We display an html form, consisting of our privacy policy (simplified for easy understanding), and a anti-request forergy token.

We load the token into the form using client-side Javascript, which prevents the token from being logged server-side (as the query would be part of the url).

### POST /verify.cgi

Body (form data):

```
action=[confirm OR block]&
token=TOKEN_SENT_VIA_EMAIL&
csrf=randomly generated token, also stored in cookie
```

If the user confirms authorization, we store their eid, service id, and public directory info (see below) in a sqlite database.

If the user blocks authorization, we store ONLY their eid in order to prevent future verification emails.

### GET /lookup.cgi?service=discord&service_id=[discord id]

Publicly accessible API, to look up basic directory info from our sqlite database.

Response: 

```
{
    "affiliation": ["student"],
    "school": ["College of Natural Sciences"],
    "major": ["Computer Science, Entry-Level"]
}
```

