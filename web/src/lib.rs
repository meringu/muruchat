#![allow(non_snake_case)]

use der::Document;
use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use pem_rfc7468::LineEnding;
use pkcs8::{EncodePrivateKey, DecodePrivateKey, spki::EncodePublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};

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
      },
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
    self.private_key.to_pkcs8_der().unwrap().to_pem(LineEnding::LF).unwrap()
  }

  pub fn public_key_pem(&self) -> String {
    self.public_key.to_public_key_der().unwrap().to_pem(LineEnding::LF).unwrap()
  }

  pub fn save(&self) {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let pem = self.private_key.to_pkcs8_der().unwrap().to_pem(LineEnding::LF).unwrap();
    storage.set("private_key", &pem).unwrap();
  }
}

pub fn app(cx: Scope) -> Element {
  cx.use_hook(|_| {
    cx.provide_context(User::from_context());
  });

  cx.render(rsx!{
      Router {
          Route { to: "/", Home {} },
          Route { to: "/chats/:other_pub", Chat {} },
      }
  })
}

fn Home(cx: Scope) -> Element {
  let user = cx.use_hook(|_| cx.consume_context::<Option<User>>());

  let generating = use_state(&cx, || false );
  let disabled = true;
  let text_visiblity = match generating.get() {
    true => "invisible",
    false => "",
  };

  cx.render(rsx!(
    div {
      class: "text-center pt-16 md:pt-32",
		  h1 {
        class: "font-bold text-3xl md:text-5xl",
        "Welcome to MuruChat"
      }
    }
    match user {
      Some(Some(user)) => rsx!(
        KeyViewer {
          private_key_pem: user.private_key_pem(),
          public_key_pem: user.public_key_pem()
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
              generating.set(true);

              let u = User::generate();
              u.save();
              web_sys::window().unwrap().location().reload().unwrap();
            },
            generating.then(|| rsx!(
              div {
                class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2",
                SpinIcon {}
              }
            ))
            div {
              class: "{text_visiblity}",
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
  let user = cx.use_hook(|_| cx.consume_context::<Option<User>>());

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
          match user {
            Some(Some(u)) => {
              u.delete();
              web_sys::window().unwrap().location().reload().unwrap();
            },
            _ => {},
          }
        },
        "delete key"
      }
    }
    show_public_key.get().then(||
      rsx!(KeyInspector { pem: private_key_pem.to_string() })
    )
    show_private_key.get().then(||
      rsx!(KeyInspector { pem: public_key_pem.to_string() })
    )
  ))
}

#[inline_props]
fn Contacts(cx: Scope) -> Element {
  cx.render(rsx! {
    div {
      class: "pt-4 md:pt-8 ml-2 md:ml-8",
		  h2 {
        class: "font-bold text-xl md:text-3xl",
        "Contacts"
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
  cx.render(rsx!{
    div {
      class: "flex justify-center mt-4 md:mt-8",
      div {
        class: "bg-gray-300 rounded p-2 font-mono",
        pem.split('\n').map(|l| rsx!(
          div { "{l}" }
        ))
      }
    }
  })
}

fn Chat(cx: Scope) -> Element {
  cx.render(rsx! (
      div { "Chat" }
  ))
}

fn SpinIcon(cx: Scope) -> Element {
  cx.render(rsx!(
    svg {
      class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white",
      xmlns: "http://www.w3.org/2000/svg",
      fill: "none",
      view_box: "0 0 24 24",
      circle {
        class: "opacity-25",
        cx: "12",
        cy: "12",
        r: "10",
        stroke: "currentColor",
        stroke_width: "4"
      }
      path {
        class: "opacity-75",
        fill: "currentColor",
        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      }
    }
  ))
}
