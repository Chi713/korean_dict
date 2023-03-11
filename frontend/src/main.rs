use yew::prelude::*;
use yew_router::prelude::*;
use components::file_uploader::FileUploader;

mod components;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}


#[function_component(Home)]
fn home() -> Html {

    html! {
        <div>
            <h1>{"korean dictionary tool"}</h1>
            <FileUploader></FileUploader>

        </div>
    }
}
