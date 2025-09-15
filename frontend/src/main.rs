use dioxus::prelude::*;
use dioxus::launch;

fn app() -> Element {
    rsx! {
        div { "Hello dioxus world!" }
    }
}

fn main() {
    launch(app);
}
