use ldap3::result::Result;
use ldap3::{LdapConn, Scope, SearchEntry};

fn main() -> Result<()> {
    let mut ldap = LdapConn::new("ldap://directory.utexas.edu:389")?;
    let (rs, _res) = ldap
        .search(
            "dc=directory,dc=utexas,dc=edu",
            Scope::Subtree,
            "sn=Aubin",
            vec!["*"],
        )?
        .success()?;
    print!("Content-type: text/html\n\n");
    println!("<html>");
    println!("<body>");
    println!("<p>");
    for entry in rs {
        println!("{:?}", SearchEntry::construct(entry));
    }
    println!("</p>");
    println!("</body>");
    println!("</html>");
    Ok(ldap.unbind()?)
}
