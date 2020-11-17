#![recursion_limit = "1024"]

#[macro_use]
extern crate lazy_static;

mod data;
mod player;

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::{IntervalService, Task};
use yew::services::storage::{Area, StorageService};
use yew::services::fetch::{FetchService, Request, Response, FetchTask};
use log::error;
use std::time::Duration;
use yew::format::{Nothing, Json};
use anyhow::Error;
use data::*;
use player::Player;

struct Model {
  link: ComponentLink<Self>,
  timeout: u64,
  timeout_enable: bool,
  items: Vec<RedditItem>,
  current_index: i32,
  url: String,
  job: Option<Box<dyn Task>>,
  callback_tick: Callback<()>,
  callback_items: Callback<()>,
  loading: bool,
  failed: bool,
  after: Option<String>,
  ft: Option<FetchTask>,
  storage: Option<StorageService>,
}

enum Msg {
  TimeoutToggle,
  TimeoutSet(ChangeData),
  SetIndex(i32),
  NextPicture,
  Tick,
  PrevPicture,
  ItemsFailed,
  ItemsLoaded((Vec<RedditItem>, String)),
  LoadItems,
}

const TIMEOUT_KEY: &str = "TIMEOUT_KEY";
const TIMEOUT_ENABLED_KEY: &str = "TIMEOUT_ENABLED_KEY";
const LIMIT: usize = 50;

impl Model {
  pub fn get_items(&mut self) -> yew::services::fetch::FetchTask {
    let callback = self.link.callback(
      move |response: Response<Json<Result<RedditListings, Error>>>| {
        let (meta, Json(data)) = response.into_parts();
        if meta.status.is_success() {
          match data {
            Ok(data) => {
              if let Some(data) = data.get_items() {
                Msg::ItemsLoaded(data)
              } else {
                Msg::ItemsFailed
              }
            }
            Err(e) => {
              error!("{}", e);
              Msg::ItemsFailed
            }
          }
        } else {
          Msg::ItemsFailed
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
    let pictures_left = self.items.len() - self.current_index as usize;
    if LIMIT / 3 > pictures_left {
      self.callback_items.emit(());
    }
  }

  fn refresh_interval(&mut self) {
    self.job.take();
    if self.timeout_enable {
      let handle = IntervalService::spawn(Duration::from_secs(self.timeout), self.callback_tick.clone());
      self.job = Some(Box::new(handle));
    }
  }

  fn check_bounds(&mut self) {
    if self.current_index < 0 {
      self.current_index = 0;
    }
    if self.current_index as usize >= self.items.len() {
      self.current_index = self.items.len() as i32 - 1;
    }
  }
}

impl Component for Model {
  type Message = Msg;
  type Properties = ();
  fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
    let window: web_sys::Window = web_sys::window().expect("window not available");
    let location = window.location();
    let mut timeout = 10;
    let mut timeout_enabled = true;
    let handle = IntervalService::spawn(Duration::from_secs(timeout), link.callback(|_| Msg::Tick));
    link.callback(|_| Msg::LoadItems).emit(());
    let initial_vec = Vec::new();
    let storage = StorageService::new(Area::Local).ok();
    if let Some(storage) = &storage {
      if let Ok(timeout_enabled_val) = storage.restore(TIMEOUT_ENABLED_KEY) {
        if let Ok(timeout_enabled_val) = timeout_enabled_val.parse::<bool>() {
          timeout_enabled = timeout_enabled_val;
        }
      }

      if let Ok(timeout_val) = storage.restore(TIMEOUT_KEY) {
        if let Ok(timeout_val) = timeout_val.parse::<u64>() {
          timeout = timeout_val;
        }
      }
    }

    Self {
      timeout,
      timeout_enable: timeout_enabled,
      items: initial_vec,
      current_index: 0,
      url: location.pathname().unwrap(),
      job: Some(Box::new(handle)),
      callback_tick: link.callback(|_| Msg::Tick),
      callback_items: link.callback(|_| Msg::LoadItems),
      link,
      loading: false,
      failed: false,
      after: None,
      ft: None,
      storage,
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::TimeoutToggle => {
        self.timeout_enable = !self.timeout_enable;
        if let Some(storage) = &mut self.storage {
          storage.store(TIMEOUT_ENABLED_KEY, Ok(self.timeout_enable.to_string()));
        }
        self.refresh_interval();
      }
      Msg::TimeoutSet(data) => {
        if let ChangeData::Value(timeout_str) = data {
          if let Ok(timeout) = timeout_str.parse::<u64>() {
            if timeout != self.timeout && timeout > 0 {
              self.timeout = timeout;
              if let Some(storage) = &mut self.storage {
                storage.store(TIMEOUT_KEY, Ok(self.timeout.to_string()));
              }
              self.refresh_interval();
            }
          }
        }
      }
      Msg::NextPicture => {
        self.current_index += 1;
        self.check_bounds();
        self.check_next_load();
        self.refresh_interval();
      }
      Msg::PrevPicture => {
        self.current_index -= 1;
        self.check_bounds();
        self.check_next_load();
        self.refresh_interval();
      }
      Msg::SetIndex(index) => {
        self.current_index = index;
        self.check_bounds();
        self.check_next_load();
        self.refresh_interval();
      }
      Msg::Tick => {
        if !self.timeout_enable {
          return false;
        }
        self.current_index += 1;
        self.check_bounds();
        self.check_next_load();
      }
      Msg::ItemsLoaded((pictures, after)) => {
        self.loading = false;
        self.after = Some(after);
        self.items.extend(pictures);
      }
      Msg::ItemsFailed => {
        self.loading = false;
        self.failed = true;
      }
      Msg::LoadItems => {
        if !self.loading {
          self.loading = true;
          self.ft.take();
          self.ft = Some(self.get_items());
        }
      }
    }
    true
  }

  fn change(&mut self, _props: Self::Properties) -> ShouldRender {
    false
  }

  fn view(&self) -> Html {
    let view_item = |item: &RedditItem| {
      match &item.item {
        RedditItemType::Picture { source_set, url } => {
          html! {
              <>
                <div id="main-title"><a target="_blank" href=format!("{}", &item.title_url)>{ &item.title }</a></div>
                <img id="main-image" src={ url } srcset={ source_set } loading="lazy" sizes="100vw" />
              </>
          }
        }
        RedditItemType::Video { url, mime } => {
          html! {
              <>
                <div id="main-title"><a target="_blank" href=format!("{}", &item.title_url)>{ &item.title }</a></div>
                <Player id="main-video" url={ url } mime={ mime } />
              </>
          }
        }
        RedditItemType::Embed { url, width, height, scrolling } => {
          html! {
              <>
                <div id="main-title"><a target="_blank" href=format!("{}", &item.title_url)>{ &item.title }</a></div>
                <div id="main-iframe-center">
                  <iframe id="main-iframe" src={url} width={width} height={height} scrolling={scrolling} border=0 frameborder=0 allowfullscreen=true
                   sandbox="allow-forms allow-same-origin allow-scripts allow-top-navigation-by-user-activation allow-popups allow-popups-to-escape-sandbox allow-orientation-lock allow-presentation"></iframe>
                 </div>
              </>
          }
        }
      }
    };

    let tool_box_number_view = |item: (usize, &RedditItem)| {
      let index = item.0 as i32;
      if self.current_index == index {
        html! { <li><a class="item-selected" onclick=self.link.callback(move |_| Msg::SetIndex(index))>{ index + 1 }</a></li> }
      } else {
        html! { <li><a onclick=self.link.callback(move |_| Msg::SetIndex(index))>{ index + 1 }</a></li> }
      }
    };
    let item = self.items.get(self.current_index as usize);
    html! {
            <div id="main">
                {
                  if self.loading {
                    html!{ <div class="loader"></div> }
                  } else if self.failed {
                    html!{ <div>{"Failed to load next index "}<a href="javascript:void(0)" onclick=self.link.callback(|_| Msg::LoadItems)>{"Retry"}</a></div> }
                  } else {
                    html!{ <></> }
                  }
                }
                {
                  if let Some(item) = item.map(view_item) {
                     item
                  } else{
                    html!{ <></> }
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
                            {for self.items.iter().enumerate().map(tool_box_number_view)}
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