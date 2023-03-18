use yew::prelude::*;
use yew_router::prelude::*;
//use yew::Properties;
use components::file_uploader::{
    Entry,
    FileUploader,
    RecievedData,
};
//use components::file_uploader2::FileUploader2;

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
    let sent_from_file_uploader = use_state(RecievedData::default);
    let sentences_cb = {
        let sent_from_file_uploader = sent_from_file_uploader.clone();
        Callback::from(move |data: RecievedData|{
            log::debug!("sent from file uploader component: {:?}", data.sentence);
            sent_from_file_uploader.set(data);
    })};

    /*
    let sentences = vec![
        Sentence {
            id: 1,
            text: "안녕하세요 테스트입니다".to_string(),
        },
        Sentence {
            id: 2,
            text: "안녕하세요, 이번 테스트입니다".to_string(),
        }
    ];
    */
    html! {
        <div>
            <h1>{"korean dictionary tool"}</h1>
            //<FileUploader sentences={sentences.clone()}/>
            <FileUploader {sentences_cb}/>
            <div>
            {
                (*sent_from_file_uploader).clone().sentence.into_iter().map(|entry: Entry|{
                    html!{<div>{ format!("entry: word: {}, definition: {:?}, explaination: {:?}", entry.word, entry.definition, entry.explaination) }</div>}
                }).collect::<Html>()
            }
            //<SentenceList sentences={sentences.clone()}/>
            </div>
        </div>
    }
}

/*
#[derive(Clone, PartialEq)]
pub struct Sentence {
    id: usize,
    text: String,
}

#[derive(Clone, PartialEq, Properties)]
pub struct SentenceListProps {
    sentences: Vec<Sentence>,
}
*/
/*
#[function_component(SentenceList)]
pub fn sentence_list(SentenceListProps {sentences} : &SentenceListProps) -> Html {
    sentences.iter().map(|sentence| html! {
        <p>{format!("{}: {}", sentence.id, sentence.text)}</p>
    }).collect()
}
*/
