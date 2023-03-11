use reqwest::blocking::Client;
use spinoff::{spinners, Color, Spinner};
use std::io::Write;
use std::{fs, io, process};

const OPENAI_API_URL: &str = "https://api.openai.com/v1";
const CHATGPT_MODEL: &str = "gpt-3.5-turbo";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct Payload {
    model: String,
    messages: Vec<Message>,
}

#[derive(serde::Deserialize)]
struct Choice {
    message: Message,
}

#[derive(serde::Deserialize)]
struct Error {
    message: String,
}

#[derive(serde::Deserialize)]
struct Response {
    error: Option<Error>,
    choices: Option<Vec<Choice>>,
}

fn prompt(client: &Client, api_key: &str, message_history: &mut Vec<Message>) -> bool {
    print!("> ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line.");

    if input.trim() == "" {
        return false;
    }

    let message = Message {
        role: "user".to_string(),
        content: input,
    };

    message_history.push(message);

    let spinner = Spinner::new(spinners::Circle, "Thinking...", Color::White);

    let response = client
        .post(format!("{}/chat/completions", OPENAI_API_URL))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&Payload {
            model: CHATGPT_MODEL.to_string(),
            messages: message_history.to_vec(),
        })
        .send()
        .expect("Failed to send request.");

    spinner.clear();

    let mut response_messages = match response
        .json::<Response>()
        .expect("Failed to parse response.")
    {
        Response {
            choices: Some(choices),
            ..
        } => choices
            .iter()
            .map(|choice| choice.message.clone())
            .collect::<Vec<Message>>(),

        Response {
            error: Some(error), ..
        } => {
            println!("Error: {}", error.message);
            process::exit(1);
        }

        _ => {
            println!("Error: No response.");
            process::exit(1);
        }
    };

    response_messages.iter().for_each(|message| {
        println!("< {}", message.content.trim());
    });

    message_history.append(&mut response_messages);

    true
}

fn main() {
    println!("Ask me a question.");

    let client = Client::new();

    let mut message_history: Vec<Message> = Vec::new();

    let api_key_path = directories::UserDirs::new()
        .expect("Failed to get user directories.")
        .home_dir()
        .join(".openai-key");

    let api_key = fs::read_to_string(api_key_path).expect("Failed to read API key.");

    while prompt(&client, &api_key.trim(), &mut message_history) {}
}
