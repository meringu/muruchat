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
                        secret_key: u.secret_key().to_string(),
                        public_key: u.public_key().to_string(),
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
fn KeyViewer(cx: Scope, secret_key: String, public_key: String) -> Element {
    let user = use_read(&cx, USER);
    let set_user = use_set(&cx, USER);

    let address_book = use_read(&cx, ADDRESS_BOOK);
    let set_address_book = use_set(&cx, ADDRESS_BOOK);

    let chats = use_read(&cx, CHATS);
    let set_chats = use_set(&cx, CHATS);

    let show_public_key = use_state(&cx, || false);
    let show_secret_key = use_state(&cx, || false);

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
                    class: "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full flex",
                    onclick: |_| {
                        if !*show_public_key.get() {
                            show_secret_key.set(false);
                        }
                        show_public_key.set(!show_public_key.get());
                    },
                    match *show_public_key.get() {
                        true => {
                            rsx!(
                                span {
                                    class: "w-6 h-6",
                                    ChevronUpIcon {}
                                }
                                "hide public key"
                            )
                        },
                        false => {
                            rsx!(
                                span {
                                    class: "w-6 h-6",
                                    ChevronDownIcon {}
                                }
                                "show public key"
                            )
                        },
                    }
                }
                button {
                    class: "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full flex",
                    onclick: |_| {
                        if !*show_secret_key.get() {
                            show_public_key.set(false);
                        }
                        show_secret_key.set(!show_secret_key.get());
                    },
                    match *show_secret_key.get() {
                        true => {
                            rsx!(
                                span {
                                    class: "w-6 h-6",
                                    ChevronUpIcon {}
                                }
                                "hide secret key"
                            )
                        },
                        false => {
                            rsx!(
                                span {
                                    class: "w-6 h-6",
                                    ChevronDownIcon {}
                                }
                                "show secret key"
                            )
                        },
                    }
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

                            chats.delete();
                            set_chats(Chats::default());
                        }
                    },
                    "clear session"
                }
            }
            show_public_key.get().then(||
                rsx!(KeyInspector { pem: public_key.to_string() })
            )
            show_secret_key.get().then(||
                rsx!(KeyInspector { pem: secret_key.to_string() })
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
    let chats = use_read(&cx, CHATS);
    let address_book = use_read(&cx, ADDRESS_BOOK);

    cx.render(rsx!(
        Container {
            h2 {
                class: "font-bold text-xl md:text-3xl",
                "Chats"
            }
            ul {
                class: "pt-4 md:pt-8 space-y-4",
                chats.none().then(|| rsx!(
                    li {
                        "No chats yet!"
                    }
                ))
                chats.iter().map(|(id, chat)| {
                    let title: String = chat.iter().map(|public_key| {
                        address_book.who_is(public_key).unwrap_or_else(|| "Unknown".to_string())
                    }).collect::<Vec<String>>().join(", ");

                    rsx!(
                        li {
                            div {
                                class: "flex space-x-2",
                                div {
                                    "{title}"
                                }
                                Link {
                                    class: "text-blue-600 hover:text-blue-700 font-bold flex",
                                    to: "/chats/{id}",
                                    "Go to chat"
                                    div {
                                        class: "h-6 w-6",
                                        ChevronRightIcon {}
                                    }
                                }
                            }
                        }
                    )
                })
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
