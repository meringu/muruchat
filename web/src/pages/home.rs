use dioxus::prelude::*;
use dioxus_router::{use_router, Link};

use crate::{components::*, state::*};

pub fn Home(cx: Scope) -> Element {
    let user = use_read(&cx, USER);

    cx.render(rsx!(
        div {
            class: "text-center pt-16 md:pt-32",
            h1 {
                class: "font-bold text-3xl md:text-5xl",
                "Welcome to MuruChat"
            }
        }
        match user {
            Some(u) => rsx!(
                div {
                    class: "pt-8 md:pt-16 space-y-8 md:space-y-16 m-2 md:m-8",
                    KeyViewer {
                        private_key_pem: u.private_key_pem(),
                        public_key_pem: u.public_key_pem()
                    }
                    Contacts { }
                    Chats { }
                }
            ),
            _ => rsx!(
                Welcome {}
            ),
        }
    ))
}

fn Welcome(cx: Scope) -> Element {
    let set_user = use_set(&cx, USER);

    cx.render(rsx!(
        div {
            class: "text-center pt-8 md:pt-16",
            p {
                "Generate or upload a private key to start chatting. (generating may take some time...)"
            }
        }
        div {
            class: "flex justify-center space-x-8 pt-2 md:pt-4",
            button {
                class: "relative bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
                onclick: move |_| {
                    let u = User::generate();
                    u.save();
                    set_user(Some(u));
                },
                div {
                    "generate key"
                }
            }
            button {
                class: "bg-blue-400 text-white font-bold py-2 px-4 rounded-full",
                "upload key"
            }
        }
    ))
}

#[inline_props]
fn KeyViewer(cx: Scope, private_key_pem: String, public_key_pem: String) -> Element {
    let user = use_read(&cx, USER);
    let set_user = use_set(&cx, USER);

    let address_book = use_read(&cx, ADDRESS_BOOK);
    let set_address_book = use_set(&cx, ADDRESS_BOOK);

    let show_public_key = use_state(&cx, || false);
    let show_private_key = use_state(&cx, || false);

    let public_key_button_text = if *show_public_key.get() {
        "hide public key"
    } else {
        "show public key"
    };

    let private_key_button_text = if *show_private_key.get() {
        "hide private key"
    } else {
        "show private key"
    };

    cx.render(rsx! (
        div {
            div {
                class: "text-center",
                p {
                    "Your encryption key is loaded"
                }
            }
            div {
                class: "flex justify-center space-x-8 mt-2 md:mt-4",
                button {
                    class: "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
                    onclick: |_| {
                        if !*show_public_key.get() {
                            show_private_key.set(false);
                        }
                        show_public_key.set(!show_public_key.get());
                    },
                    "{public_key_button_text}"
                }
                button {
                    class: "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
                    onclick: |_| {
                        if !*show_private_key.get() {
                            show_public_key.set(false);
                        }
                        show_private_key.set(!show_private_key.get());
                    },
                    "{private_key_button_text}"
                }
                button {
                    class: "bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-full",
                    onclick: move |_| {
                        let ok = web_sys::window().unwrap().confirm_with_message(
                            "Are you sure you want to delete your encryption key, contact, and chat lists? This won't delete messages in MuruChat, but you will need to add your key, contacts, and chats back to see any messages."
                        ).unwrap();
                        if ok {
                            if let Some(u) = user {
                                u.delete();
                            }
                            set_user(None);

                            address_book.delete();
                            set_address_book(AddressBook::default());
                        }
                    },
                    "clear session"
                }
            }
            show_public_key.get().then(||
                rsx!(KeyInspector { pem: public_key_pem.to_string() })
            )
            show_private_key.get().then(||
                rsx!(KeyInspector { pem: private_key_pem.to_string() })
            )
        }
    ))
}

fn Contacts(cx: Scope) -> Element {
    let router = use_router(&cx);

    let address_book = use_read(&cx, ADDRESS_BOOK);
    let chats = use_read(&cx, CHATS);
    let set_chats = use_set(&cx, CHATS);

    let visible_count = 5;

    let show_more = address_book.len() > visible_count;

    cx.render(rsx! {
        Container {
            div {
                class: "flex space-x-2",
                h2 {
                    class: "font-bold text-xl md:text-3xl",
                    "Contacts"
                }
                Link {
                    class: "h-6 w-6 text-blue-600",
                    to: "/contacts/add",
                    PlusIcon {}
                }
            }
            ul {
                class: "pt-4 md:pt-8 space-y-4",
                address_book.iter().take(5).map(|(public_key, nickname)| {
                    rsx!(
                        li {
                            div {
                                class: "flex space-x-2",
                                div {
                                    "{nickname}"
                                }
                                button {
                                    class: "text-blue-600 hover:text-blue-700 font-bold flex",
                                    onclick: move |_| {
                                        let mut new_chats = chats.clone();

                                        let new_chat = Chat::from_public_key(public_key.clone());
                                        let chat_id = new_chat.id();

                                        // skip creating chat if it already exists.
                                        if new_chats.add_chat(chat_id.clone(), new_chat).is_ok() {
                                            new_chats.save();
                                            set_chats(new_chats);
                                        }
                                        router.push_route(&format!("/chats/{}", chat_id), None, None);
                                    },
                                    "Start chat"
                                    div {
                                        class: "h-6 w-6",
                                        ChevronRightIcon {}
                                    }
                                }
                            }
                        }
                    )
                })
                show_more.then(||
                    rsx!(
                        li { "Show more contacts..." }
                    )
                )
            }
        }
    })
}

fn Chats(cx: Scope) -> Element {
    cx.render(rsx!(
        Container {
            h2 {
                class: "font-bold text-xl md:text-3xl",
                "Chats"
            }
            div {
                class: "pt-4 md:pt-8",
                "No chats yet!"
            }
        }
    ))
}

#[inline_props]
fn KeyInspector(cx: Scope, pem: String) -> Element {
    cx.render(rsx!(
        div {
            class: "flex justify-center mt-4 md:mt-8",
            button {
                class: "m-2 h-6 w-6",
                onclick: move |_| {
                    #[allow(unused_must_use)] {
                        web_sys::window().unwrap()
                            .navigator()
                            .clipboard().unwrap()
                            .write_text(pem);
                    }
                },
                DuplicateIcon {}
            }
            div {
                class: "flex bg-gray-300 rounded",
                div {
                    class: "p-2 font-mono",
                    pem.split('\n').map(|l| rsx!(
                        div { "{l}" }
                    ))
                }
            }
        }
    ))
}
