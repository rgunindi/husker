use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use dioxus::prelude::*;
use dioxus_cli_config::*;
use http::Method;
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowHeaders, Any, CorsLayer};
mod server {
    include!("../server/db_utils.rs");
}
use server::*;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Word {
    no_sentence: String,
    hidden_sentence: String,
    tr_sentence: String,
    no_word: String, // Base form (infinitive) - "forlate"
    tr_word: String,
    conjugated_word: String, // Conjugated form in sentence - "forlot"
    correct_attempts: i32,   // Track successful attempts
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/game")]
    WordGame {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us

    use server_fn::client::set_server_url;

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
        ])
        .allow_origin(Any);
    let mut address = fullstack_address_or_localhost();

    // Set up the axum router
    let router = axum::Router::new().layer(cors).register_server_functions();
    // This will add a fallback route to the router that will serve your component and server functions
    // .serve_dioxus_application(ServeConfigBuilder::default(), App);

    // Try binding to the address, if fails try alternative ports
    let listener = loop {
        match tokio::net::TcpListener::bind(address).await {
            Ok(listener) => {
                println!("Server started successfully on {}", address);
                break listener;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    // Try next port
                    let new_port = address.port() + 1;
                    address.set_port(new_port);
                    println!("Port in use, trying port {}", new_port);
                    continue;
                }
                panic!("Failed to bind to address: {}", e);
            }
        }
    };

    let router = router.into_make_service();
    println!("Local axum server running on {}", address);
    let ip = "https://backoffice.koyeb.app";
    // let ip2 = "http://192.168.1.109:8080";
    // set_server_url(ip);
    info!("SERVERIP:{0}", server_fn::client::get_server_url());
    println!("SERVERIP:{0}", server_fn::client::get_server_url());
    axum::serve(listener, router).await.unwrap();
}
#[cfg(not(feature = "server"))]
fn main() {
    use dioxus::{
        fullstack::prelude::server_fn::client::{get_server_url, set_server_url},
        logger::tracing::info,
    };
    if get_server_url().is_empty() {
        println!("IP ADRESI BOS!!");
        let server_ip = std::env::var("ip")
            .unwrap_or_else(|_| "https://infodonnamobil.onrender.com".to_string())
            .leak();
        let ip = "https://backoffice.koyeb.app";

        let _serverurl = format!(
            "{ip}:{}",
            std::env::var("PORT").unwrap_or_else(|_| "8080".to_string())
        )
        .leak();

        // let ip2 = "https://infodonnamobil.onrender.com";
        // set_server_url(server_ip);
        // set_server_url("http://192.168.1.109:8080");
        info!("SERVERIP:{0}", get_server_url());
        println!("SERVERIP:{0}", get_server_url());
    }
    info!("SERVERIP:{0}", get_server_url());
    println!("SERVERIP:{0}", get_server_url());
    dioxus::launch(App);
}
#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}
/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        div { class: "home",
            h1 { "Welcome to Norwegian Word Practice" }
            p { "Practice your Norwegian vocabulary with our interactive learning tool." }
            Link { class: "start-button", to: Route::WordGame {}, "Start Practice" }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::WordGame {} ,
                "Practice"
            }
        }

        Outlet::<Route> {}
    }
}

/// WordGame component
#[component]
fn WordGame() -> Element {
    let word_set = use_signal(Word::new_set);
    let mut current_word_index = use_signal(|| 0);

    // Fetch words from database
    use_effect(move || {
        to_owned![word_set];
        spawn(async move {
            if let Ok(content) = get_words_set().await {
                word_set.set(Word::set(content));
            }
        });
    });

    // Get current word from word_set
    let current_word = use_memo(move || {
        if word_set().is_empty() {
            // Return default word if word_set is empty
            Word::new()
        } else {
            // Get word at current index, wrapping around if needed
            word_set()[current_word_index() % word_set().len()].clone()
        }
    });

    let mut user_input = use_signal(String::new);
    let mut show_answer = use_signal(|| false);
    let message = use_signal(String::new);

    let check_answer = move |_| async move {
        to_owned![message, show_answer, current_word_index, user_input];

        // Compare case-insensitive
        if user_input().to_lowercase() == current_word().no_word.to_lowercase() {
            let mut msg = String::from("Correct! Press Enter for next word.");
            msg.push_str("</br>");
            // Show the full sentence when the answer is checked
            let setning = format!(
                "<p class=\"no_sentence\">{}</p>",
                &current_word().no_sentence
            );
            msg.push_str(&setning);
            message.set(msg.into());

            // Update the correct attempts in the database
            update_word_attempts(current_word().no_word.clone()).await;

            user_input.set("".into());
            show_answer.set(true);
        } else {
            message.set("Incorrect. Try again.".into());
            show_answer.set(true);
        }
    };

    rsx! {
            div {
                class: "word-game",

                // Norwegian sentence
                div { class: "sentence",
                    h3 { "Fill in the missing word:" }
                    p { "{current_word().hidden_sentence}" }
                }

                // Translation
                div { class: "translation",
                    p { "Translation: {current_word().tr_sentence}" }
                }
     // Tips
     div { class: "Tips:",
     p { "Tips: {current_word().tr_word}" }
    }
                // Input area
                div { class: "input-area",
                    input {
                        // disabled:"{show_answer}",
                        value: "{user_input}",
                        oninput: move |evt| user_input.set(evt.value()),
                        onkeydown: move |evt|async move {
                            if evt.key() == keyboard_types::Key::Enter {
                                check_answer(()).await;
                            }
                        }
                    }
                    button { onclick: move |_| check_answer(()),id:"check", "Check" }
                }
                 // New "Continue" button for mobile users
                 button { onclick: move |_| {
                    // Only increment the index if the answer was correct
                    if show_answer() {
                        current_word_index.set(current_word_index() + 1);
                        user_input.set("".into());
                        show_answer.set(false);
                    }
                }, id: "continue", "Continue" }


                // Message area
                if !message().is_empty() {
                    p { class: "message", dangerous_inner_html:"{message}" }
                }

                // Show answer when needed
                if show_answer() {
                    p { class: "answer",
                        "Correct word: "
                        span { class: "highlight", "{current_word().no_word}" }
                    }
                }
            }
        }
}

impl Word {
    fn new() -> Self {
        Self {
            no_sentence: "default".to_string(),
            hidden_sentence: "default".to_string(),
            tr_sentence: "default".to_string(),
            no_word: "default".to_string(),
            tr_word: "default".to_string(),
            conjugated_word: "default".to_string(),
            correct_attempts: 0,
        }
    }
    fn new_set() -> Vec<Self> {
        let default = Self::new();
        vec![default]
    }
    fn set(ws: Vec<WordSet>) -> Vec<Self> {
        let mut word_set: Vec<Self> = vec![];
        for i in ws {
            // Create sentence with hidden word by replacing the conjugated form
            let hidden_sentence = if let Some(word_index) = i
                .no_sentence
                .to_lowercase()
                .find(&i.conjugated_word.to_lowercase())
            {
                let underscores = "_".repeat(i.conjugated_word.len());
                let mut sentence = i.no_sentence.clone();
                sentence.replace_range(
                    word_index..word_index + i.conjugated_word.len(),
                    &underscores,
                );
                sentence
            } else {
                format!("{} (___)", i.no_sentence)
            };

            word_set.push(Word {
                no_sentence: i.no_sentence,
                hidden_sentence,
                tr_sentence: i.tr_sentence,
                no_word: i.no_word,
                tr_word: i.tr_word,
                conjugated_word: i.conjugated_word,
                correct_attempts: i.correct_attempts,
            });
        }
        word_set
    }
}
