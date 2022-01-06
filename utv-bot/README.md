# UTexas Verify Discord Bot

Users interact with this bot to verify and connect their account to their student electronic ID.

### Behaviors
1. Verified users will have a `✓` at the end of their nickname on all servers that have this bot active.
2. If a user in a guild has a `✓` in their nickname, this nickname will be set to their username with any
existing `✓`'s replaced with `_`.
3. This bot watches for new members joining the guild and any updates to a guild member's name.
4. Verified users will have a `UTexas Verified` role added

### Gateway Intents
 * `GATEWAY_MEMBERS`: necessary to access when a user enters a guild and when they change their nicks.
 * `DIRECT_MESSAGES`: to receive the token that will be DM'ed by the user

### Server Permissions
 * Create Slash Commands
 * Manage Roles: allows bot to create the `UTexas Verified` role and assign it to members
 * Manage Members: allows modification of nicknames

### Commands
`/verify eid:str`
The user enters their EID and an email will be sent to the address they have on file in the UT Directory.
They will receive a token in the email which they must DM to this bot to finish connecting their account.

`/help`