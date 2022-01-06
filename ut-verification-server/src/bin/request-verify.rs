use utv_server::directory::Person;
use utv_token;

fn main() {
    let shared_key = std::env::var("SHARED_KEY").expect("Missing SHARED_KEY");
    let encryption_key = std::env::var("ENCRYPTION_KEY").expect("Missing ENCRYPTION_KEY");
    let sendgrid_api_key = std::env::var("SENDGRID_API_KEY").expect("Missing SENDGRID_API_KEY");

    let shared_key =
        base64::decode_config(shared_key, base64::URL_SAFE_NO_PAD).expect("Invalid ENCRYPTION_KEY");
    let encryption_key = base64::decode_config(encryption_key, base64::URL_SAFE_NO_PAD)
        .expect("Invalid ENCRYPTION_KEY");

    // TODO: authorize incoming requests
    let mut eid = String::new();
    if let Err(_) = std::io::stdin().read_line(&mut eid) {
        println!("Status: 400 Bad Request");
        return;
    }

    let person = match Person::lookup(eid.trim(), &encryption_key) {
        Ok(person) => person,
        Err(_) => {
            println!("Status: 400 Bad Request");
            return;
        }
    };

    let email = format!("{}@eid.utexas.edu", eid.trim());

    let req = serde_json::json!({
        "from": {
            "email": "no-reply@verifiedbot.com"
        },
        "personalizations": [
            {
                "to": [
                    {
                        "email": email
                    }
                ],
                "dynamic_template_data": {
                    "name": person.name,
                    "token": utv_token::encode_token(&person.claims, &shared_key)
                }
            }
        ],
        "template_id": "d-0c3e84eccf584d07ae3035bdf0a81716"
    })
    .to_string();

    let res = reqwest::blocking::Client::new()
        .post("https://api.sendgrid.com/v3/mail/send")
        .bearer_auth(sendgrid_api_key)
        .header("Content-Type", "application/json")
        .body(req)
        .send()
        .unwrap();

    let status = res.status();

    if status.is_success() {
        println!("Status: 204 No Response");
    } else {
        println!("Status: 500 Internal Server Error");
        eprintln!("{:#?}", res);
        eprintln!("{:#?}", res.json::<serde_json::Value>());
    }
}
