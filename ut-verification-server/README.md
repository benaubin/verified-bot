# UT Verifcation Server

__This is now deprecated as VerifiedBot no longer uses LDAP to verify users__

Accepts UT EIDs over a sqs queue.

We then fetch basic user information over LDAP.

Send an email containing a signed token conisting of verified details about the user,
along with an encrypted eid.
