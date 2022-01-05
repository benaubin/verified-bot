use ldap3::result::Result;
use ldap3::{LdapConn, Scope, SearchEntry};

fn main() -> Result<()> {
    let eid = "no3969";

    let mut ldap = LdapConn::new("ldap://directory.utexas.edu:389")?;
    let (rs, _res) = ldap
        .search(
            "dc=directory,dc=utexas,dc=edu",
            Scope::Subtree,
            &("uid=".to_owned()+eid),
            vec!["*"],
        )?
        .success()?;

    // Convert to a json string that could be returned by a REST API
    let json : String = format!("{:?}",&SearchEntry::construct(rs.first().unwrap().to_owned()))[12..].to_string();
    println!("{}",json);

    Ok(ldap.unbind()?)
}
