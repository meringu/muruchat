#![allow(non_snake_case)]

mod components;

use der::Document;
use dioxus::prelude::*;
use dioxus_router::{use_router, Link, Route, Router};
use pem_rfc7468::LineEnding;
use pkcs8::{
    spki::{DecodePublicKey, EncodePublicKey},
    DecodePrivateKey, EncodePrivateKey,
};
use rsa::{RsaPrivateKey, RsaPublicKey};
use wasm_bindgen::JsCast;

use components::*;

static USER: Atom<Option<User>> = |_| User::from_context();
static CONTACTS: Atom<Vec<Contact>> = |_| vec![];

#[derive(Clone)]
struct User {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl User {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        Self::from_private_key(private_key)
    }

    pub fn from_context() -> Option<Self> {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        match storage.get_item("private_key").unwrap() {
            Some(private_key_pem) => {
                let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key_pem).unwrap();
                Some(Self::from_private_key(private_key))
            }
            None => None,
        }
    }

    pub fn from_private_key(private_key: RsaPrivateKey) -> Self {
        let public_key = RsaPublicKey::from(&private_key);

        Self {
            private_key,
            public_key,
        }
    }

    pub fn delete(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.clear().unwrap();
    }

    pub fn private_key_pem(&self) -> String {
        self.private_key
            .to_pkcs8_der()
            .unwrap()
            .to_pem(LineEnding::LF)
            .unwrap()
            .trim()
            .to_string()
    }

    pub fn public_key_pem(&self) -> String {
        self.public_key
            .to_public_key_der()
            .unwrap()
            .to_pem(LineEnding::LF)
            .unwrap()
            .trim()
            .to_string()
    }

    pub fn save(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let pem = self
            .private_key
            .to_pkcs8_der()
            .unwrap()
            .to_pem(LineEnding::LF)
            .unwrap();
        storage.set("private_key", &pem).unwrap();
    }
}

#[derive(Clone)]
struct Contact {
    nickname: String,
    public_key: RsaPublicKey,
}

impl Contact {
    fn new(nickname: String, public_key: RsaPublicKey) -> Self {
        Self {
            nickname,
            public_key,
        }
    }
}

pub fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            Route { to: "/", Home {} },
            Route { to: "/contacts/add", AddContact {} },
            Route { to: "/chats/:other_pub", Chat {} },
        }
    })
}

fn Home(cx: Scope) -> Element {
    let user = use_read(&cx, USER);
    let set_user = use_set(&cx, USER);

    let disabled = true;

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
        KeyViewer {
          private_key_pem: u.private_key_pem(),
          public_key_pem: u.public_key_pem()
        }
        Contacts { }
        Chats { }
      ),
      _ => rsx!(
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
            disabled: "{disabled}",
            "upload key"
          }
        }
      ),
    }
  ))
}

#[inline_props]
fn KeyViewer(cx: Scope, private_key_pem: String, public_key_pem: String) -> Element {
    let user = use_read(&cx, USER);
    let set_user = use_set(&cx, USER);

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
      class: "text-center pt-8 md:pt-16",
      p {
        "Your encryption key is loaded"
      }
    }
    div {
      class: "flex justify-center space-x-8 pt-2 md:pt-4",
      button {
        class: "relative bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
        onclick: |_| {
          if !*show_public_key.get() {
            show_private_key.set(false);
          }
          show_public_key.set(!show_public_key.get());
        },
        "{public_key_button_text}"
      }
      button {
        class: "relative bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
        onclick: |_| {
          if !*show_private_key.get() {
            show_public_key.set(false);
          }
          show_private_key.set(!show_private_key.get());
        },
        "{private_key_button_text}"
      }
      button {
        class: "relative bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-full",
        onclick: move |_| {
          if let Some(u) = user {
            u.delete();
          }
          set_user(None);
        },
        "delete key"
      }
    }
    show_public_key.get().then(||
      rsx!(KeyInspector { pem: public_key_pem.to_string() })
    )
    show_private_key.get().then(||
      rsx!(KeyInspector { pem: private_key_pem.to_string() })
    )
  ))
}

#[inline_props]
fn Contacts(cx: Scope) -> Element {
    let contacts = use_read(&cx, CONTACTS);

    cx.render(rsx! {
      div {
        class: "pt-4 md:pt-8 ml-2 md:ml-8",
        div {
          class: "flex space-x-2",
          h2 {
            class: "font-bold text-xl md:text-3xl",
            "Contacts"
          }
          Link {
            class: "h-6 w-6 text-blue-600",
            to: "/contacts/add",
            icons::Plus {}
          }
        }
        ul {
          class: "pt-4 md:pt-8 space-y-4",
          contacts.iter().map(|contact| rsx!(
            li { "{contact.nickname}" }
          ))
        }
      }
    })
}

#[inline_props]
fn Chats(cx: Scope) -> Element {
    cx.render(rsx! {
      div {
        class: "pt-4 md:pt-8 ml-2 md:ml-8",
            h2 {
          class: "font-bold text-xl md:text-3xl",
          "Chats"
        }
      }
    })
}

#[inline_props]
fn KeyInspector(cx: Scope, pem: String) -> Element {
    cx.render(rsx!(
      div {
        class: "flex justify-center mt-4 md:mt-8",
        div {
          class: "flex bg-gray-300 rounded",
          div {
            class: "p-2 font-mono",
            pem.split('\n').map(|l| rsx!(
              div { "{l}" }
            ))
          }
          div {
            class: "p-2",
            button {
              class: "h-6 w-6",
              onclick: move |_| {
                #[allow(unused_must_use)] {
                  web_sys::window().unwrap()
                    .navigator()
                    .clipboard().unwrap()
                    .write_text(pem);
                }
              },
              icons::Duplicate {}
            }
          }
        }
      }
    ))
}

fn Chat(cx: Scope) -> Element {
    cx.render(rsx! (
        div { "Chat" }
    ))
}

fn AddContact(cx: Scope) -> Element {
    let router = use_router(&cx);

    let contacts = use_read(&cx, CONTACTS);
    let set_contacts = use_set(&cx, CONTACTS);

    cx.render(rsx!(
    div {
      class: "flex justify-center m-2 md:m-8",
      div {
        class: "pt-4 md:pt-8 space-y-8",
        div {
          h2 {
            class: "font-bold text-xl md:text-3xl",
            "Add Contact"
          }
        }
        div {
          p {
            "Enter your contacts public key, and assign them a nickname to start chatting"
          }
        }
        div {
          label {
            class: "block text-gray-700 text-sm font-bold mb-2",
            r#for: "nickname",
            "Nickname"
          }
          input {
            class: "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline",
            id: "nickname",
            r#type: "text",
            placeholder: "Enter a nickname",
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
            class: "shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline font-mono",
            id: "public_key_pem",
            placeholder: "-----BEGIN PUBLIC KEY-----\n\n...\n\n-----END PUBLIC KEY-----"
          }
        }
        div {
          class: "flex justify-end",
          button {
            class: "relative bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
            onclick: move |_| {
              let nickname = web_sys::window().unwrap()
                .document().unwrap()
                .get_element_by_id("nickname").unwrap()
                .dyn_into::<web_sys::HtmlInputElement>().unwrap()
                .value();

              let public_key_pem = web_sys::window().unwrap()
                .document().unwrap()
                .get_element_by_id("public_key_pem").unwrap()
                .dyn_into::<web_sys::HtmlTextAreaElement>().unwrap()
                .value();

              let public_key = RsaPublicKey::from_public_key_pem(&public_key_pem).unwrap();

              let contact = Contact::new(nickname, public_key);

              set_contacts(contacts.iter().chain([&contact]).cloned().collect());

              router.push_route("/", None, None);
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
