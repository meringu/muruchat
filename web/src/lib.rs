#![allow(non_snake_case)]

pub mod components {
    mod container;
    mod icons;

    pub use container::*;
    pub use icons::*;
}

pub mod pages {
    mod add_contact;
    mod chat;
    mod home;

    pub use add_contact::*;
    pub use chat::*;
    pub use home::*;
}

mod state {
    mod address_book;
    mod chats;
    mod user;

    pub use address_book::*;
    pub use chats::*;
    pub use user::*;
}

use dioxus::prelude::*;
use dioxus_router::{Route, Router};

use crate::pages::*;

pub fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            Route { to: "/", Home {} },
            Route { to: "/contacts/add", AddContact {} },
            Route { to: "/chats/:other_pub", Chat {} },
        }
    })
}
