use yew::prelude::*;
use yew::Properties;
use std::iter::zip;
use crate::{Entry, components::file_uploader::RecievedData};

#[derive(Clone, PartialEq, Properties)]
pub struct SentenceViewerProps {
    pub recieved_data: RecievedData,
}

#[function_component(SentenceViewer)]
pub fn sentence_viewer(props : &SentenceViewerProps) -> Html {
    let props = &props.recieved_data;
    let selected_word = use_state_eq(|| None);
    let selected_def = use_state_eq(|| None);

    let select_def_cb = {
        let selected_word = selected_word.clone();
        let selected_def = selected_def.clone();

        Callback::from(move |(word_index, def_index): (usize, usize)| {
            selected_word.set(Some(word_index));
            selected_def.set(Some(def_index));
    })};

    let selection_preview = match (*selected_word,*selected_def) {
        (Some(w_index), Some(d_index)) => {
            let entry = &props.sentence_entries[w_index];
            html!{
                <span>{format!("selected word: {} | selected def: {}", entry.word, entry.definition[d_index])}</span>
            }
        },
        _ => html!{<span>{"No Selection"}</span>}
    };

    html! {
        <>
            <NavButtons 
                prev_state={props.prev_flag} 
                next_state={props.next_flag} 
            ></NavButtons>
            <div id="preview-sentence">{&props.sentence}</div>
            
            <div>{selection_preview}</div>
            <div id="entry-list">
            
            <ol class="entry">
                <EntryList 
            sentence_entries={props.sentence_entries.clone()} 
            word_index={*selected_word}
            select_def_cb={select_def_cb}></EntryList>
            </ol>
            <NavButtons 
                prev_state={props.prev_flag} 
                next_state={props.next_flag} 
            ></NavButtons>
            </div>

        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct NavButtonsProp {
    pub prev_state: bool,
    pub next_state: bool,
}

#[function_component(NavButtons)]
fn nav_buttons(props: &NavButtonsProp) -> Html {
    html!{
            <section>
                <div>
                    if props.prev_state {
                        <button>{"Prev"}</button>
                    }
                    if props.next_state {
                        <button>{"Next"}</button>
                    }
                </div>
            </section>
        
    }
}

#[derive(Properties, PartialEq, Clone)]
struct EntryListProp {
    sentence_entries: Vec<Entry>,
    word_index: Option<usize>,
    select_def_cb: Callback<(usize,usize)>
}

#[function_component(EntryList)]
fn entry_list (props: &EntryListProp) -> Html {

    props
        .clone()
        .sentence_entries.into_iter()
        .enumerate()
        .map(|(entry_index, entry): (usize, Entry)|{
            let select_def_cb = props.select_def_cb.clone();
            let mut is_selected: bool = false;
            if let Some(b) = props.word_index {
                if b == entry_index {
                    is_selected = true;
                }
            }
            html!{
                <li key={entry.word.clone()}>
                    <strong class="entry-word">{&entry.word}</strong>
                            
                    <ol>
                        <DefList {entry} {entry_index} {is_selected} {select_def_cb}></DefList>
                    </ol>
                </li>
            }
        }).collect::<Html>()
}

#[derive(Properties, PartialEq, Clone)]
struct DefListProp {
    pub entry: Entry,
    pub entry_index: usize,
    pub is_selected: bool,
    pub select_def_cb: Callback<(usize, usize)>,
}

#[function_component(DefList)]
fn entry_list(props: &DefListProp) -> Html {
    let def_index_state: UseStateHandle<Option<usize>> = use_state(Option::default);

    let def_index_cb = {
        let props = props.clone();
        let def_index_state = def_index_state.clone();
        Callback::from(move |def_index: usize| {
            props.select_def_cb.emit((props.entry_index, def_index));
            def_index_state.set(Some(def_index));
    })};

    let definition = props.entry.definition.clone();
    let explaination = props.entry.explaination.clone();

    zip(definition, explaination)
        .enumerate()
        .map(|(def_index, (def,expl))| {
            let def_index_state =def_index_state.clone();
            let mut selected_class = String::default();
            if let Some(s) = *def_index_state {
                if def_index == s && props.is_selected {
                    selected_class = "selected".to_string();
                }
            }

            html!{
            <li>
                <div class={classes!("definition", selected_class)}
                    onclick={
                        let def_index_cb = def_index_cb.clone();
                        Callback::from(move |_| {
                            def_index_cb.emit(def_index);
                        })
                    }
                >
                    <div>{def}</div>
                    <div class="explaination">{expl}</div>
                </div>
            </li>
        }}).collect::<Html>()
}
