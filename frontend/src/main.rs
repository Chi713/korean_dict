use yew::prelude::*;
use yew_router::prelude::*;
use components::file_uploader::{
    Entry,
    FileUploader,
    RecievedData,
};
use components::sentence_viewer::SentenceViewer;

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
    let file_uploader_data = use_state(RecievedData::default);
    let sentences_cb = {
        let file_uploader_data = file_uploader_data.clone();
        Callback::from(move |data: RecievedData|{
            log::debug!("sent from file uploader component: {:?}\n{:#?}", data.sentence, data.sentence_entries);
            file_uploader_data.set(data);
    })};

    html! {
        <div>
            <h1 id="title">{"korean dictionary tool"}</h1>
            if (*file_uploader_data) == RecievedData::default() {
            <FileUploader {sentences_cb}/>
            }else {
            <SentenceViewer
                sentence={(*file_uploader_data).clone().sentence}
                sentence_entries={(*file_uploader_data).clone().sentence_entries}
            />}
        </div>
    }
}
