use yew::{Component, ComponentLink, ShouldRender, Html, Properties};
use yew::prelude::*;
use web_sys::HtmlVideoElement;
use wasm_bindgen::JsCast;

#[derive(Properties, Clone, PartialEq)]
pub struct PlayerProps {
  pub url: String,
  pub mime: String,
  pub id: String,
}

pub enum Msg {
  Enable,
  OnLoad(Event),
}

pub struct Player {
  props: PlayerProps,
  dirty: bool,
  callback_enable: Callback<()>,
  link: ComponentLink<Self>,
}

impl Component for Player {
  type Properties = PlayerProps;
  type Message = Msg;
  fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
    Self {
      props,
      dirty: false,
      callback_enable: link.callback(|_| Msg::Enable),
      link,
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::Enable => {
        if self.dirty {
          self.dirty = false;
          return true;
        }
      }
      Msg::OnLoad(event) => {
        if let Some(target) = event.target() {
          if let Some(video) = target.dyn_ref::<HtmlVideoElement>() {
            video.set_muted(true);
          }
        }
      }
    }
    false
  }

  fn change(&mut self, props: Self::Properties) -> ShouldRender {
    if self.props != props {
      self.props = props;
      if !self.dirty {
        self.dirty = true;
        self.callback_enable.emit(());
      }
      true
    } else {
      false
    }
  }

  fn view(&self) -> Html {
    if !self.dirty {
      html! {
         <video id={&self.props.id} autoplay=true loop=true muted=true onloadeddata=self.link.callback(|x| Msg::OnLoad(x))>
              <source src={ &self.props.url } type={ &self.props.mime } />
         </video>
      }
    } else {
      html! {
         <></>
      }
    }
  }
}