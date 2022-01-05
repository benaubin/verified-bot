use ldap3::result::Result;
use ldap3::{LdapConn, Scope, SearchEntry};

fn main() {
    println!("{}", get_ldap_user_json("no3969").unwrap());
    println!("\n\n");
    println!("{}", get_ldap_user_json("bha366").unwrap());
}

fn get_ldap_user_json(eid: &str) -> Result<String> {
    let mut ldap = LdapConn::new("ldap://directory.utexas.edu:389")?;
    let (rs, _res) = ldap
        .search(
            "dc=directory,dc=utexas,dc=edu",
            Scope::Subtree,
            &format!("uid={}", eid),
            vec!["*"],
        )?
        .success()?;

    // Convert to a json string that could be returned by a REST API
    let json : String = format!("{:?}",&SearchEntry::construct(rs.first().unwrap().to_owned()))[12..].to_string();

   return Ok(json);
}