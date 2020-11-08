#![recursion_limit = "1024"]

mod data;

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::{IntervalService, Task};
use yew::services::fetch::{FetchService, Request, Response, FetchTask};
use log::{info, error};
use std::time::Duration;
use yew::format::{Nothing, Json};
use anyhow::Error;
use std::collections::HashMap;
use data::*;


struct Model {
  link: ComponentLink<Self>,
  timeout: u64,
  timeout_enable: bool,
  pictures: Vec<RedditPicture>,
  current_index: i32,
  url: String,
  job: Option<Box<dyn Task>>,
  callback_tick: Callback<()>,
  callback_pictures: Callback<()>,
  loading: bool,
  failed: bool,
  after: Option<String>,
  ft: Option<FetchTask>,
}

enum Msg {
  TimeoutToggle,
  TimeoutSet(ChangeData),
  SetIndex(i32),
  NextPicture,
  Tick,
  PrevPicture,
  PicturesFailed,
  PicturesLoaded((Vec<RedditPicture>, String)),
  LoadPictures,
}

const LIMIT: usize = 50;
impl Model {
  pub fn get_images(&mut self) -> yew::services::fetch::FetchTask {
    let callback = self.link.callback(
      move |response: Response<Json<Result<RedditListings, Error>>>| {
        let (meta, Json(data)) = response.into_parts();
        info!("META: {:?}, {:?}", meta, data);
        if meta.status.is_success() {
          match data {
            Ok(data) => {
              if let Some(data) = data.get_images() {
                Msg::PicturesLoaded(data)
              } else {
                Msg::PicturesFailed
              }
            }
            Err(e) => {
              error!("{}", e);
              Msg::PicturesFailed
            }
          }
        } else {
          Msg::PicturesFailed
        }
      },
    );

    let request_url = if let Some(after) = &self.after {
      format!("https://www.reddit.com/{}/.json?limit={}&after={}", &self.url, LIMIT, after)
    } else {
      format!("https://www.reddit.com/{}/.json?limit={}", &self.url, LIMIT)
    };

    let request = Request::get(&request_url).body(Nothing).unwrap();
    FetchService::fetch(request, callback).unwrap()
  }

  fn check_next_load(&self) {
    if self.loading {
      return;
    }
    let pictures_left = self.pictures.len() - self.current_index as usize;
    if LIMIT / 3 > pictures_left {
      self.callback_pictures.emit(());
    }
  }
}

impl Component for Model {
  type Message = Msg;
  type Properties = ();
  fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
    let window: web_sys::Window = web_sys::window().expect("window not available");
    let location = window.location();
    let timeout = 10;
    let handle = IntervalService::spawn(Duration::from_secs(timeout), link.callback(|_| Msg::Tick));
    link.callback(|_| Msg::LoadPictures).emit(());
    let initial_vec = Vec::new();
    Self {
      timeout: 10,
      timeout_enable: true,
      pictures: initial_vec,
      current_index: 0,
      url: location.pathname().unwrap(),
      job: Some(Box::new(handle)),
      callback_tick: link.callback(|_| Msg::Tick),
      callback_pictures: link.callback(|_| Msg::LoadPictures),
      link,
      loading: false,
      failed: false,
      after: None,
      ft: None,
    }
  }


  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::TimeoutToggle => {
        self.timeout_enable = !self.timeout_enable;
        self.job.take();
        if self.timeout_enable {
          let handle = IntervalService::spawn(Duration::from_secs(self.timeout), self.callback_tick.clone());
          self.job = Some(Box::new(handle));
        }
      }
      Msg::TimeoutSet(data) => {
        if let ChangeData::Value(timeout_str) = data {
          if let Ok(timeout) = timeout_str.parse::<u64>() {
            if timeout != self.timeout && timeout > 0 {
              self.timeout = timeout;
              self.job.take();
              let handle = IntervalService::spawn(Duration::from_secs(self.timeout), self.callback_tick.clone());
              self.job = Some(Box::new(handle));
            }
          }
        }
      }
      Msg::NextPicture => {
        self.current_index += 1;
        if self.current_index as usize >= self.pictures.len() {
          self.current_index = self.pictures.len() as i32 - 1;
        }
        self.check_next_load();

        self.job.take();
        let handle = IntervalService::spawn(Duration::from_secs(self.timeout), self.callback_tick.clone());
        self.job = Some(Box::new(handle));
      }
      Msg::PrevPicture => {
        self.current_index -= 1;
        if self.current_index < 0 {
          self.current_index = 0;
        }
        self.check_next_load();

        self.job.take();
        let handle = IntervalService::spawn(Duration::from_secs(self.timeout), self.callback_tick.clone());
        self.job = Some(Box::new(handle));
      }
      Msg::SetIndex(index) => {
        self.current_index = index;
        if self.current_index < 0 {
          self.current_index = 0;
        }
        if self.current_index as usize >= self.pictures.len() {
          self.current_index = self.pictures.len() as i32 - 1;
        }
        self.check_next_load();

        self.job.take();
        let handle = IntervalService::spawn(Duration::from_secs(self.timeout), self.callback_tick.clone());
        self.job = Some(Box::new(handle));
      }
      Msg::Tick => {
        if !self.timeout_enable {
          return false;
        }

        self.current_index += 1;
        if self.current_index as usize >= self.pictures.len() {
          self.current_index = self.pictures.len() as i32 - 1;
        }
        self.check_next_load();
      }
      Msg::PicturesLoaded((pictures, after)) => {
        self.loading = false;
        self.after = Some(after);
        self.pictures.extend(pictures);
      }
      Msg::PicturesFailed => {
        self.loading = false;
        self.failed = true;
      }
      Msg::LoadPictures => {
        if !self.loading {
          self.loading = true;
          self.ft = Some(self.get_images());
        }
      }
    }
    true
  }

  fn change(&mut self, _props: Self::Properties) -> ShouldRender {
    false
  }

  fn view(&self) -> Html {
    let view_image = |image: &RedditPicture| {
      html! {
          <>
            <div id="main-title"><a target="_blank" href=format!("{}", &image.title_url)>{ &image.title }</a></div>
            <img id="main-image" src=format!("{}", &image.url) srcset=format!("{}", image.source_set)  loading="lazy" sizes="100vw" />
          </>
       }
    };

    let tool_box_number_view = |item: (usize, &RedditPicture)| {
      let index = item.0 as i32;
      if self.current_index == index {
        html! { <li><a class="item-selected" onclick=self.link.callback(move |_| Msg::SetIndex(index))>{ index + 1 }</a></li> }
      } else {
        html! { <li><a onclick=self.link.callback(move |_| Msg::SetIndex(index))>{ index + 1 }</a></li> }
      }
    };
    let image = self.pictures.get(self.current_index as usize);
    html! {
            <div id="main">
                {
                  if self.loading {
                    html!{ <div class="loader"></div> }
                  } else if self.failed {
                    html!{ <div>{"Failed to load next index"} <a onclick=self.link.callback(|_| Msg::LoadPictures)>{"Retry"}</a></div> }
                  } else {
                    html!{ <></> }
                  }
                }
                {
                  if let Some(image) = image.map(view_image) {
                     image
                  } else{
                    html!{ <div>{"No image"}</div> }
                  }
                }
                <div class="prev-button" onclick=self.link.callback(|_| Msg::PrevPicture)></div>
                <div class="next-button" onclick=self.link.callback(|_| Msg::NextPicture)></div>
                <div class="toolbox">
                    <div class="toolbox-header">
                      <strong class="reddit-name">{ &self.url }</strong><br/>
                      <input type="checkbox" checked={self.timeout_enable} onchange=self.link.callback(|_| Msg::TimeoutToggle) /> <strong>{"Auto next"}</strong>{" every"}
                      <input type="number" class="number-input" value={self.timeout} onchange=self.link.callback(|data| Msg::TimeoutSet(data)) /> {"seconds"}
                    </div>
                    <div class="toolbox-body">
                        <ul>
                            {for self.pictures.iter().enumerate().map(tool_box_number_view)}
                        </ul>
                    </div>
                </div>
            </div>
    }
  }
}

#[wasm_bindgen(start)]
pub fn run_app() {
  wasm_logger::init(wasm_logger::Config::default());
  App::<Model>::new().mount_to_body();
}