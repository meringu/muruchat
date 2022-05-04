use dioxus::prelude::*;

#[inline_props]
pub fn Container<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    cx.render(rsx!(
        div {
            class: "pt-4 pb-8 md:pt-8 pl-2 pr-2 md:pl-8 md:pr-8 max-w-7xl mx-auto border rounded-lg shadow",
            children
        }
    ))
}
