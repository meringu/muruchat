use dioxus::prelude::*;
use dioxus_router::{use_route, Link};

use muruchat::{handshake::Challenge, pki::{PublicKey, SecretKey}};

use wasm_bindgen::JsCast;

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
        Tester {
            chat_id: chat_id
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

#[inline_props]
fn Tester<'a>(cx: Scope, chat_id: &'a str) -> Element {
    let user = use_read(&cx, USER);

    cx.render(rsx!(
        div {
            class: "flex justify-center",
            button {
                class: "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full",
                onclick: move |_| {
                    if let Some(u) = user {
                        if let Err(e) = do_ws(chat_id, &u.public_key(), &u.secret_key()) {
                            web_sys::console::log_1(&e);
                        }
                    }
                },
                div {
                    "test websocket"
                }
            }
        }
    ))
}

#[derive(Debug)]
enum FSM {
    WaitingForChallenge,
    Authed,
}

fn do_ws<'a>(_chat_id: &'a str, public_key: &PublicKey, secret_key: &SecretKey) -> Result<(), wasm_bindgen::JsValue> {
    // open connection
    let ws = web_sys::WebSocket::new("ws://127.0.0.1:8787/chat")?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let cloned_pk = public_key.clone();
    let cloned_sk = secret_key.clone();

    // create callback
    let cloned_ws = ws.clone();

    let mut fsm = FSM::WaitingForChallenge;

    let onmessage_callback = wasm_bindgen::prelude::Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
        // Only care about array buffers
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            web_sys::console::log_1(&format!("message event, received arraybuffer: {:?}", abuf).into());

            let array = js_sys::Uint8Array::new(&abuf);
            
            web_sys::console::log_1(&format!("processing fsm: {:?}", fsm).into());

            let next = match fsm {
                FSM::WaitingForChallenge => {
                    let sig = match Challenge::from_bytes(&array.to_vec()) {
                        Ok(c) => c.sign(&cloned_sk),
                        Err(e) => {
                            web_sys::console::error_1(&format!("{:?}", e).into());
                            return;
                        }
                    };

                    if let Err(e) = cloned_ws.send_with_u8_array(sig.bytes()) {
                        web_sys::console::error_1(&e);
                        return;
                    };

                    if let Err(e) = cloned_ws.send_with_u8_array("World".as_bytes()) {
                        web_sys::console::error_1(&e);
                        return;
                    };

                    FSM::Authed
                },
                FSM::Authed => {
                    if let Ok(s) = String::from_utf8(array.to_vec()) {
                        web_sys::console::log_1(&s.into());
                    }
                    FSM::Authed
                },
            };

            fsm = next;

            // let array = js_sys::Uint8Array::new(&abuf);
            // let len = array.byte_length() as usize;
            // web_sys::console::log_1(&format!("Arraybuffer received {}bytes: {:?}", len, array.to_vec()).into());
            // // here you can for example use Serde Deserialize decode the message
            // // for demo purposes we switch back to Blob-type and send off another binary message
            // cloned_ws.set_binary_type(web_sys::BinaryType::Blob);
            // match cloned_ws.send_with_u8_array(&vec![5, 6, 7, 8]) {
            //     Ok(_) => web_sys::console::log_1(&format!("binary message successfully sent").into()),
            //     Err(err) => web_sys::console::log_1(&format!("error sending message: {:?}", err).into()),
            // }
        }
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);

    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    // on error callback just prints the error to the console
    let onerror_callback = wasm_bindgen::prelude::Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
        web_sys::console::error_1(&e);
    }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // on open call back to send public key to start handshake
    let cloned_ws = ws.clone();
    let onopen_callback = wasm_bindgen::prelude::Closure::wrap(Box::new(move |_| {
        if let Err(e) = cloned_ws.send_with_u8_array(&cloned_pk.bytes()) {
            web_sys::console::error_1(&e);
        };
    }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}
