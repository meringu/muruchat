use dioxus::prelude::*;
use dioxus_router::Link;

use crate::components::*;

pub fn Chat(cx: Scope) -> Element {
    let chat_title = "Chat with X";

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
                        "{chat_title}"
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
