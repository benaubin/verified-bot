use super::deterministic_aes;
use ldap3::{Scope, SearchEntry};
use utv_token::VerifiedClaims;

#[derive(Debug)]
pub struct Person {
    pub claims: VerifiedClaims,
    pub name: String,
    pub email: Option<String>,
}


impl Person {
    pub async fn lookup(ldap: &mut ldap3::Ldap, eid: &str, encryption_key: &[u8]) -> Result<Person, LookupError> {
        let results = ldap
            .search(
                "dc=directory,dc=utexas,dc=edu",
                Scope::Subtree,
                &format!("uid={}", eid),
                vec!["*"],
            )
            .await?;

        let mut entry = results
            .0
            .into_iter()
            .map(|r| SearchEntry::construct(r))
            .find(|e| match e.attrs.get("utexasEduPersonEid") {
                Some(eids) => eids[0] == eid,
                _ => false,
            })
            .ok_or(LookupError::NotFound)?;

        let encrypted_eid = deterministic_aes::encrypt(eid.as_bytes(), encryption_key);

        let claims = VerifiedClaims {
            encrypted_eid,
            major: entry
                .attrs
                .remove("utexasEduPersonMajor")
                .ok_or(LookupError::MissingDirectoryInfo("major"))?,
            school: entry
                .attrs
                .remove("utexasEduPersonSchool")
                .ok_or(LookupError::MissingDirectoryInfo("school"))?,
            affiliation: entry
                .attrs
                .remove("utexasEduPersonPubAffiliation")
                .ok_or(LookupError::MissingDirectoryInfo("affiliation"))?,
        };

        let person = Person {
            claims,
            name: entry
                .attrs
                .remove("displayName")
                .and_then(|a| a.into_iter().next())
                .ok_or(LookupError::MissingDirectoryInfo("name"))?,
            email: entry
                .attrs
                .remove("mail")
                .and_then(|a| a.into_iter().next()),
        };

        Ok(person)
    }
}

#[derive(Debug)]
pub enum LookupError {
    MissingDirectoryInfo(&'static str),
    NotFound,
    LdapError(ldap3::LdapError),
}

impl From<ldap3::LdapError> for LookupError {
    fn from(err: ldap3::LdapError) -> Self {
        Self::LdapError(err)
    }
}
