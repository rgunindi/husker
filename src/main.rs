use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Word {
    no_sentence: String,
    tr_sentence: String,
    no_word: String,
    tr_word: String,
    correct_attempts: i32, // Track successful attempts
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
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
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

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Hero {}
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
    let current_word = use_signal(|| Word {
        no_sentence: "De ___ huset i all hast.".into(),
        tr_sentence: "Aceleyle evi terk ettiler.".into(),
        no_word: "forlot".into(),
        tr_word: "terk etmek".into(),
        correct_attempts: 0,
    });

    let mut user_input = use_signal(String::new);
    let show_answer = use_signal(|| false);
    let message = use_signal(String::new);

    let check_answer = move |_| async move {
        to_owned![message, show_answer];

        if user_input() == current_word().no_word {
            message.set("Correct! Press Enter for next word.".into());
            // Here you would update the word's correct_attempts in MongoDB
            show_answer.set(false);
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
                p { "{current_word().no_sentence}" }
            }

            // Translation
            div { class: "translation",
                p { "Translation: {current_word().tr_sentence}" }
            }

            // Input area
            div { class: "input-area",
                input {
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

            // Message area
            if !message().is_empty() {
                p { class: "message", "{message}" }
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
