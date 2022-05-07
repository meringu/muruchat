use dioxus::prelude::*;
use dioxus_router::{use_route, Link};

use crate::{components::*, state::*};

pub fn Chat(cx: Scope) -> Element {
    let chats = use_read(&cx, CHATS);
    let address_book = use_read(&cx, ADDRESS_BOOK);

    let route = use_route(&cx);

    let chat_id = route.last_segment().unwrap();

    let chat = match chats.get(chat_id) {
        Some(c) => c,
        None => return cx.render(rsx!(
            div {
                class: "flex justify-center m-2 md:m-8",
                div {
                    class: "pt-4 md:pt-8 space-y-8",
                    div {
                        class: "flex",
                        Link {
                            class: "h-6 w-6 md:h-8 md:w-8 inline lg:left-8 lg:top-8 lg:absolute text-blue-600",
                            to: "/",
                            ArrowLeftIcon {}
                        }
                        h2 {
                            class: "font-bold text-xl md:text-3xl pl-2 md:pl-4 lg:pl-0",
                            "Chat not found"
                        }
                    }
                }
            }
        )),
    };

    let title: String = chat
        .iter()
        .map(|public_key| {
            address_book
                .who_is(public_key)
                .unwrap_or_else(|| "Unknown".to_string())
        })
        .collect::<Vec<String>>()
        .join(", ");

    cx.render(rsx!(
        div {
            class: "flex justify-center m-2 md:m-8",
            div {
                class: "pt-4 md:pt-8 space-y-8",
                div {
                    class: "flex",
                    Link {
                        class: "h-6 w-6 md:h-8 md:w-8 inline lg:left-8 lg:top-8 lg:absolute text-blue-600",
                        to: "/",
                        ArrowLeftIcon {}
                    }
                    h2 {
                        class: "font-bold text-xl md:text-3xl pl-2 md:pl-4 lg:pl-0",
                        "{title}"
                    }
                }
            }
        }
        Container {
            div {
                "..."
            }
            div {
                "..."
            }
            div {
                "..."
            }
        }
    ))
}
