use yew::prelude::*;
use yew::{
    Callback, 
    Component, 
    Context, 
    html::TargetCast, 
    Html,
    platform::spawn_local,
};

use web_sys::{
    Event,
    FileList,
    HtmlInputElement
};

use gloo::file::{
    callbacks::FileReader,
    File,
};

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

//-----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Entry {
    pub word: String,
    pub definition: Vec<String>,
    pub explaination: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RecievedData {
    sentence: Vec<Entry>,
    prev_flag: bool,
    next_flag: bool,
}

#[derive(Clone, PartialEq)]
struct FileDetails {
    name: String,
    data: String,
}


pub enum Msg {
    Loaded(String, String),
    Files(Vec<File>),
    Sent,
    Recieved(AttrValue),
    Clear,
}

pub struct FileUploader {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
    file_return_data: RecievedData,
}

//------------------------------------------------------------------------------

impl Component for FileUploader {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {

        Self {
            readers: HashMap::default(),
            files: Vec::default(),
            file_return_data: RecievedData::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, data) => {
                self.files.clear();
                self.files.push(FileDetails {
                    data,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);

                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_text(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                //file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }

                true
            }
            Msg::Sent => {
                
                if !self.files.is_empty() {
                    log::debug!("file sent to server");
                    let upload_file_cb = ctx.link().callback(Msg::Recieved);
                    Self::upload_file(self.files.clone(), upload_file_cb);
                }

                false
            }
            Msg::Recieved(data) => {
                let json_data: RecievedData = serde_json::from_str(&data).unwrap();
                self.file_return_data = json_data.clone();
                log::info!("server response:\n{:?}", json_data);

                true
            }
            Msg::Clear => {
                self.files.clear();

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let onchange = {ctx.link().callback(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Self::on_file_change(input.files())
        })};

        let ondrop=ctx.link().callback(|event: DragEvent| {
            event.prevent_default();
            let files = event.data_transfer().unwrap().files();
            Self::on_file_change(files)
        });

        let onclick = ctx.link().callback(move |_| { Msg::Sent });
 
        let clear_file = ctx.link().callback(move |_| { Msg::Clear });

        html! {
            <div id="wrapper">
                
                <p id="title">{ "Upload Your Files To The Cloud" }</p>
                
                <label for="file-upload">
                    <div
                        id="drop-container"
                        ondrop={ondrop}
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
                    {onchange}
                />
                <div>
                    <button {onclick} >{"Upload"}</button>
                    <button onclick ={clear_file}>{"clear selected file"}</button>
                </div>
                <div id="preview-area">
                    { for self.files.iter().map(Self::view_file) }
                </div>
            </div>
        }
    }
}

//---------------------------------------------------------------------------------------------------------

impl FileUploader {
    fn view_file(file: &FileDetails) -> Html {
        html! {
            <div class="preview-tile">
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div>
                    {format!("{}", &file.data)}
                </div>
                
            </div>
        }
    }

    fn on_file_change(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        //log::debug!("file result{:?}",result);
        Msg::Files(result)
    }

    fn upload_file(files: Vec<FileDetails>, upload_file_cb: Callback<AttrValue>) {
        spawn_local(async move {
            //log::debug!("in the async section now\n\n");
            let mut file_json = HashMap::new();
            let files = files.to_owned();
            file_json.insert("name", &files[0].name);
            file_json.insert( "data", &files[0].data);

            let client = reqwest::Client::new();
            //TODO fix to use a url variable instead of hard coded value
            let server_res = client.post("http://localhost:8080/api/fileData")
                .json(&file_json)
                .send()
                .await
                .unwrap();
            let res_body = server_res.text().await.unwrap();
            upload_file_cb.emit(AttrValue::from(res_body));
        });
    }
}

