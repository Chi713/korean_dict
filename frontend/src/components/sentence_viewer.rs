//use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::Properties;
use std::iter::zip;
use crate::Entry;

#[derive(Clone, PartialEq, Properties)]
pub struct SentenceViewerProps {
    pub sentence: String,
    pub sentence_entries: Vec<Entry>,
}
#[function_component(SentenceViewer)]
pub fn sentence_viewer(props : &SentenceViewerProps) -> Html {
    let selected_word = use_state_eq(|| None);
    let selected_def = use_state_eq(|| None);

    let select_def_cb = {
        let props = props.clone();
        let selected_word = selected_word.clone();
        let selected_def = selected_def.clone();

        Callback::from(move |(word_index, def_index): (usize, usize)| {
            let entry = props.sentence_entries[word_index].clone();
            log::debug!("triggered from select_def_cb!! {} | {}\n indexes:{word_index}|{def_index}", entry.word, entry.definition[def_index]);
            selected_word.set(Some(word_index));
            selected_def.set(Some(def_index));
    })};

    html! {
        <>
            <div id="preview-sentence">{&props.sentence}</div>
            
            <section>
            <div>
            if let Some(w_index) = *selected_word {
                <span>{format!("selected word: {}", props.sentence_entries[w_index].word)}</span>
                if let Some(d_index) = *selected_def {
                    <span>{format!(" | selected def: {}", props.sentence_entries[w_index].definition[d_index])}</span>
                }
            }
            </div>
            
            <ol class="entry">
            {props.clone().sentence_entries.into_iter().enumerate().map(|(index, entry): (usize, Entry)|{
                html!{
                    <li key={entry.word.clone()}>
                        <strong class="entry-word">{&entry.word}</strong>
                            
                        <ol>
                                <EntryList entry={entry} entry_index={index} select_def_cb={select_def_cb.clone()}></EntryList>
                        </ol>
                    </li>
                }
            }).collect::<Html>()}
            </ol>
            </section>
            <section>
                <div>
                    <button>{"Prev"}</button>
                    <button>{"Next"}</button>
                </div>
            </section>

        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct EntryListProp {
    pub entry: Entry,
    pub entry_index: usize,
    pub select_def_cb: Callback<(usize, usize)>,
}

#[function_component(EntryList)]
fn entry_list(props: &EntryListProp) -> Html {

    zip(props.entry.definition.clone(), props.entry.explaination.clone())
        .enumerate()
        .map(|(def_index, (def,expl))| {
            let props = props.clone();
            html!{
            <li>
                <div class="definition"
                    onclick={Callback::from(move |_| {props.select_def_cb.emit((props.entry_index, def_index ));})}
                >
                    <div>{def}</div>
                    <div class="explaination">{expl}</div>
                </div>
            </li>
        }}).collect::<Html>()
}
