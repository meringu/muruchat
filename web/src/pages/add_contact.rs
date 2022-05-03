use dioxus::prelude::*;
use dioxus_router::{use_router, Link};
use pkcs8::spki::DecodePublicKey;

use rsa::RsaPublicKey;

use crate::{components::*, state::*};

pub fn AddContact(cx: Scope) -> Element {
    let router = use_router(&cx);

    let user = use_read(&cx, USER);

    let address_book = use_read(&cx, ADDRESS_BOOK);
    let set_address_book = use_set(&cx, ADDRESS_BOOK);

    let nickname = use_state(&cx, || "".to_string());
    let public_key_pem = use_state(&cx, || "".to_string());

    let error = use_state(&cx, || "".to_string());
    let has_error = error.get() != "";

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
                        "Add Contact"
                    }
                }
                div {
                    p {
                        "Enter your contact's public key, and assign them a nickname to start chatting"
                    }
                }
                div {
                    label {
                        class: "block text-gray-700 text-sm font-bold mb-2",
                        r#for: "nickname",
                        "Nickname"
                    }
                    input {
                        class: "shadow border rounded w-full py-2 px-3 text-gray-700",
                        id: "nickname",
                        r#type: "text",
                        placeholder: "Enter a nickname",
                        value: "{nickname}",
                        oninput: move |evt| nickname.set(evt.value.clone())
                    }
                }
                div {
                    label {
                        class: "block text-gray-700 text-sm font-bold mb-2",
                        r#for: "public_key_pem",
                        "Public Key"
                    }
                    textarea {
                        rows: "10",
                        cols: "65",
                        class: "shadow border rounded w-full py-2 px-3 text-gray-700 font-mono",
                        id: "public_key_pem",
                        placeholder: "-----BEGIN PUBLIC KEY-----\n\n...\n\n-----END PUBLIC KEY-----",
                        value: "{public_key_pem}",
                        oninput: move |evt| public_key_pem.set(evt.value.clone())
                    }
                }
                has_error.then(|| {
                    let e = error.get();

                    rsx!(
                        p {
                            class: "text-red-600",
                            "{e}"
                        }
                    )
                })
                div {
                    class: "flex justify-end",
                    button {
                        class: "relative bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
                        onclick: move |_| {
                            if nickname.get() == "" {
                                return error.set("Please enter a nickname for the contact.".to_string());
                            }

                            if public_key_pem.get() == "" {
                                return error.set("Please enter an RSA public key for the contact.".to_string());
                            }

                            let public_key = match RsaPublicKey::from_public_key_pem(&public_key_pem) {
                                Ok(k) => k,
                                Err(_) => return error.set("Failed to parse RSA public key".to_string()),
                            };

                            if let Some(u) = user {
                                if u.public_key() == public_key {
                                    return error.set("You can't add your own public key as a contact.".to_string());
                                }
                            }

                            let mut addrs = address_book.clone();
                            match addrs.add_contact(nickname.to_string(), public_key) {
                                Ok(_) => {
                                    addrs.save();
                                    set_address_book(addrs);

                                    router.push_route("/", None, None);
                                },
                                Err(e) => error.set(e.to_string()),
                            }
                        },
                        div {
                            "Create Contact"
                        }
                    }
                }
            }
        }
    ))
}
