use std::fs;
use fantoccini::{ClientBuilder, Locator};
use fantoccini::cookies::Cookie;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut caps = serde_json::map::Map::new();
    let chrome_opts = serde_json::json!({ "args": ["--headless", "--disable-gpu"] });
    caps.insert("goog:chromeOptions".to_string(), chrome_opts.clone());
    let client = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:9515")
        .await?;

    client.goto("https://www.ipko.pl/").await?;

    client.delete_all_cookies().await?;
    let cookies_string = fs::read_to_string("cookies.txt").expect("Unable to read cookies.txt file");
    let cookies = cookies_string.split("\n").collect::<Vec<_>>();
    for x in cookies {
        let cookie = Cookie::parse(x.to_owned()).unwrap();
        client.add_cookie(cookie).await?;
    }

    if let Ok(email_field) = client.wait().for_element(Locator::XPath("//*[@name=\"view./LOGIN/.data.login\"]")).await {
        email_field.send_keys(env!("IPKO_USERNAME")).await?;

        let next_button = client.wait().for_element(Locator::XPath("//*[@data-text=\"Dalej\"]")).await?;
        next_button.click().await?;

        let password_field = client.wait().for_element(Locator::XPath("//*[@type=\"password\"]")).await?;
        password_field.send_keys(env!("IPKO_PASSWORD")).await?;

        let login_button = client.wait().for_element(Locator::XPath("//*[@data-text=\"Zaloguj\"]")).await?;
        login_button.click().await?;

        let first_funds_div = client.wait().for_element(Locator::XPath("(//div[text()='Dostępne środki']/parent::div/div[2]/div[1])[1]")).await?;
        let first_funds_text = first_funds_div.html(true).await?;

        let second_funds_div = client.wait().for_element(Locator::XPath("(//div[text()='Dostępne środki']/parent::div/div[2]/div[1])[2]")).await?;
        let second_funds_text = second_funds_div.html(true).await?;

        println!("Funds: {} zł | {} zł", first_funds_text.replace(",", ".").replace(" ", ""), second_funds_text.replace(",", ".").replace(" ", ""));

        // let funds = first_funds_text.replace(",", ".").replace(" ", "").parse::<f32>().unwrap();
        // println!("Funds: {}", funds);

        // let cookies_string = client.get_all_cookies().await?.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n");
        // fs::write("cookies.txt", cookies_string).expect("Unable to write file");

        fs::write(env!("FUNDS_FILE"), format!("{} zł", first_funds_text.replace(",", ".").replace(" ", ""))).expect("Unable to write file");
    }
    client.close().await?;

    Ok(())
}