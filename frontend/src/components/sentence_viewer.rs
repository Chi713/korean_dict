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
    html! {
        <>
            <div id="preview-sentence">{props.clone().sentence}</div>
            <ol class="entry">
            {
                props.clone().sentence_entries.into_iter().map(|entry: Entry|{
                    html!{
                        <li>
                            <strong class="entry-word">{entry.word}</strong>
                            
                            <div><ol>
                                {entry_list(entry.definition, entry.explaination)}
                            </ol></div>
                        </li>
                    }
                }).collect::<Html>()
            }
            </ol>
        </>
    }
}

fn entry_list(definition: Vec<String>, explaination: Vec<String>) -> Html {
    let thingy = zip(definition, explaination);
    
    thingy.map(|(def,expl)| html!{
            <li>
                {def}
                <div>{expl}</div>
            </li>
        }).collect::<Html>()
}
