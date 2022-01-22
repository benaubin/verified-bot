# UT Verifcation Server

Accepts UT EIDs over a sqs queue.

We then fetch basic user information over LDAP.

Send an email containing a signed token conisting of verified details about the user,
along with an encrypted eid.
