use dioxus::prelude::*;

#[inline_props]
fn Base(cx: Scope, d: String) -> Element {
    cx.render(rsx!(
      svg {
        xmlns: "http://www.w3.org/2000/svg",
        fill: "none",
        view_box: "0 0 24 24",
        stroke: "currentColor",
        stroke_width: "2",
        path {
          stroke_linecap: "round",
          stroke_linejoin: "round",
          d: "{d}"
        }
      }
    ))
}

pub fn ArrowLeftIcon(cx: Scope) -> Element {
    cx.render(rsx!(Base {
        d: "M10 19l-7-7m0 0l7-7m-7 7h18".to_string()
    }))
}

pub fn DuplicateIcon(cx: Scope) -> Element {
    cx.render(rsx!(
    Base {
      d: "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z".to_string()
    }
  ))
}

pub fn PlusIcon(cx: Scope) -> Element {
    cx.render(rsx!(Base {
        d: "M12 4v16m8-8H4".to_string()
    }))
}
