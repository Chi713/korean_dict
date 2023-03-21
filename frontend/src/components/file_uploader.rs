use yew::prelude::*;
use yew::{
    Callback, 
    html::TargetCast, 
    Html,
    platform::spawn_local,
    Properties
};

use web_sys::{
    Event,
    FileList,
    HtmlInputElement
};

use gloo::file::File;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const PORT: usize = 8080;

//-----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Entry {
    pub word: String,
    pub definition: Vec<String>,
    pub explaination: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RecievedData {
    pub sentence: String,
    pub sentence_entries: Vec<Entry>,
    pub prev_flag: bool,
    pub next_flag: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct FileDetails {
    name: String,
    data: String,
}

#[derive(Clone, PartialEq)]
pub struct Sentence {
    pub id: usize,
    pub text: String,
}

#[derive(Properties, Clone, PartialEq)]
pub struct SentenceListProps {
    pub sentences_cb: Callback<RecievedData>,
}

//------------------------------------------------------------------------------

#[function_component(FileUploader)]
pub fn file_component(props: &SentenceListProps) -> Html {
    let files = use_state(Vec::new);
    let file_data: UseStateHandle<Option<FileDetails>> = use_state(|| None);
    let readers = use_state(|| None);

    let on_file_change = {
        let files = files.clone();
        Callback::from(move |file_list: Option<FileList>| {
        let files = files.clone();
        let mut result = Vec::new();

        if let Some(file_list) = file_list {
            let file_list = js_sys::try_iter(&file_list)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(file_list);
        }
        files.set(result);
    })};

    let read_file_cb = {
        let file_data = file_data.clone();
        let readers = readers.clone();
        Callback::from(move |file_details: FileDetails| {
            file_data.set(Some(file_details));
            readers.set(None);  
    })};

    {
        let readers = readers.clone();
        use_effect_with_deps(move |files| {
            if !files.is_empty() {
                let file = files[0].clone();
                let file_name = file.name();

                let mut result = vec![];
                let task = {gloo::file::callbacks::read_as_text(&file, move |res| {
                    read_file_cb.emit(FileDetails {
                        name: file_name,
                        data: res.expect("failed to read file"),
                    });
                })};
                result.push(task);
                readers.set(Some(result)); //Must hold onto reader for inner value to evaluate 
            }
        }, files.clone());
    }

    let submit_file = {
        let file_data = file_data.clone();
        let props = props.clone();
        Callback::from(move |_| {
            let file_data = file_data.clone();
            let props = props.clone();
            spawn_local(async move {
                if (*file_data).is_some() {
                    log::debug!("file being sent to server");

                    let mut file_json:HashMap<&str, &str> = HashMap::new();
                    let file = (*file_data).to_owned().unwrap();
                    file_json.insert("name", &file.name);
                    file_json.insert( "data", &file.data);

                    let client = reqwest::Client::new();
                    //TODO fix to use a url variable instead of hard coded value
                    let url = format!("http://localhost:{}/api/fileData", PORT);
                    let server_res = client.post(url)
                        .json(&file_json)
                        .send()
                        .await
                        .unwrap();
                    let res_body = server_res.text().await.unwrap();
                    log::debug!("res_body: {}", res_body);
                    let parsed_body: RecievedData = serde_json::from_str(&res_body).unwrap(); 
                    props.sentences_cb.emit(parsed_body);
                }
            });
        })
    };
    
    html! {
        <div id="wrapper">
                
            //<p id="title">{ "Upload Your Files To The Cloud" }</p>
                
            <label for="file-upload">
                <div
                    id="drop-container"
                    ondrop={ 
                        let on_file_change = on_file_change.clone();
                        Callback::from(move |event: DragEvent| {
                            event.prevent_default();
                            let event_files = event.data_transfer().unwrap().files();
                            on_file_change.emit(event_files);
                    })}
                    ondragover={Callback::from(|event: DragEvent| {
                    event.prevent_default();
                    })}
                    ondragenter={Callback::from(|event: DragEvent| {
                        event.prevent_default();
                    })}
                >
                    <i class="fa fa-cloud-upload"></i>
                    <p>{"Drop your files here or click to select"}</p>
                </div>
            </label>
                
            <input
                id="file-upload"
                type="file"
                accept=".tsv,.csv"
                multiple={false}
                onchange={Callback::from(move |e: Event| {
                    let input: HtmlInputElement = e.target_unchecked_into(); //Option<FileList>
                    on_file_change.emit(input.files());
                })}
            />
            <div>
                <button onclick={submit_file} >{"Upload"}</button>
                <button onclick ={
                    let files = files.clone();
                    let file_data = file_data.clone();
                    let readers = readers.clone();
                    Callback::from(move |_| {
                        files.set(vec![]);
                        file_data.set(None);
                        readers.set(None);
                })}>{"clear selected file"}</button>
            </div>
            if let Some(file_data) = (*file_data).clone() {
            <div id="preview-area">
                { view_file(&file_data.clone()) }
            </div>
            }
        </div>
    }
}

fn view_file(file: &FileDetails) -> Html {
    html! {
        <div class="preview-tile">
            <p class="preview-name">{ format!("{}", file.name) }</p>
            /*
            <div>
                {format!("{}", &file.data)}
            </div>
            */
        </div>
    }
}
