use std::collections::HashMap;
use serde::Deserialize;

//TODO: make http backend to this

pub(crate) enum RedditItemType {
  Picture {
    source_set: String,
    url: String,
  },
  Video {
    mime: String,
    url: String,
  },
  Embed {
    url: String,
    scrolling: String,
    width: i32,
    height: i32,
  },
}

pub(crate) struct RedditItem {
  pub(crate) title: String,
  pub(crate) title_url: String,
  pub(crate) item: RedditItemType,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditListings {
  data: Option<RedditListingsData>,
}

fn extract_imgur_gifv_url(url: &str) -> Option<String> {
  use regex::Regex;
  lazy_static! {
      static ref RE: Regex = Regex::new(r"/((\w+|\d+)).gifv").unwrap();
  }

  return RE.captures(url)
      .and_then(|x| x.get(1))
      .map(|x| x.as_str().to_string());
}

fn extract_gfycat_gif_url(url: &str) -> Option<String> {
  use regex::Regex;
  lazy_static! {
        static ref RE: Regex = Regex::new(r"^https?://thumbs\.gfycat\.com/(\w+|\d+)-(?:\w+|\d+|_)\.gif$").unwrap();
  }
  return RE.captures(url)
      .and_then(|x| x.get(1))
      .map(|x| x.as_str().to_string());
}

fn extract_src_from_inframe_html(content: &str)  -> Option<String> {
  use regex::Regex;
  lazy_static! {
        static ref RE: Regex = Regex::new("src=\"([^\"]*)\"").unwrap();
  }
  return RE.captures(content)
      .and_then(|x| x.get(1))
      .map(|x| x.as_str().to_string());
}


impl RedditListings {
  pub(crate) fn get_items(self) -> Option<(Vec<RedditItem>, String)> {
    if let Some(data) = self.data {
      let mut items = Vec::new();
      let mut after = String::new();
      for child in data.children {
        if let Some(child_data) = child.data {
          after = child_data.name;
          if child.kind != "t3" {
            continue;
          }

          if child_data.url.contains("imgur") && child_data.url.ends_with(".gifv") {
            if let Some(url) = extract_imgur_gifv_url(&child_data.url) {
              items.push(RedditItem {
                title: child_data.title.clone(),
                title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                item: RedditItemType::Video {
                  mime: "video/mp4".to_string(),
                  url: format!("https://i.imgur.com/{}.mp4", url),
                },
              });
              continue;
            }
          }

          if let Some(RedditMedia { type_: Some(type_), oembed: Some(OEmbed { thumbnail_url: Some(thumbnail_url) }) }) = &child_data.media {
            if type_ == "gfycat.com" {
              if let Some(url) = extract_gfycat_gif_url(thumbnail_url) {
                items.push(RedditItem {
                  title: child_data.title.clone(),
                  title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                  item: RedditItemType::Video {
                    mime: "video/mp4".to_string(),
                    url: format!("https://giant.gfycat.com/{}.mp4", url),
                  },
                });
                continue;
              }
            }
          }

          if let Some(RedditMediaEmbed { scrolling: Some(scrolling), width: Some(width), height: Some(height), content: Some(content) }) = &child_data.secure_media_embed {
            if let Some(url) = extract_src_from_inframe_html(content) {
              items.push(RedditItem {
                title: child_data.title.clone(),
                title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                item: RedditItemType::Embed {
                  url: url.replace("&amp;", "&"),
                  scrolling: if *scrolling { "yes".to_string() } else { "no".to_string() },
                  width: *width,
                  height: *height,
                },
              });

              continue;
            }
          }

          if let Some(media_metadata) = child_data.media_metadata {
            if !media_metadata.is_empty() {
              for item in media_metadata {
                items.push(RedditItem {
                  title: child_data.title.clone(),
                  title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                  item: RedditItemType::Picture {
                    source_set: item.1.get_srcset(),
                    url: item.1.s.u.replace("&amp;", "&"),
                  },
                });
              }
              continue;
            }
          }
          if let Some(preview) = child_data.preview {
            if !preview.images.is_empty() {
              for item in preview.images {
                if let Some(variants) = &item.variants {
                  if let Some(mp4) = &variants.mp4 {
                    items.push(RedditItem {
                      title: child_data.title.clone(),
                      title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                      item: RedditItemType::Video {
                        mime: "video/mp4".to_string(),
                        url: mp4.source.url.replace("&amp;", "&"),
                      },
                    });
                    continue;
                  }

                  if let Some(gif) = &variants.gif {
                    items.push(RedditItem {
                      title: child_data.title.clone(),
                      title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                      item: RedditItemType::Picture {
                        source_set: gif.get_srcset(),
                        url: gif.source.url.replace("&amp;", "&"),
                      },
                    });
                    continue;
                  }
                }

                items.push(RedditItem {
                  title: child_data.title.clone(),
                  title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                  item: RedditItemType::Picture {
                    source_set: item.get_srcset(),
                    url: item.source.url.replace("&amp;", "&"),
                  },
                });
              }
              continue;
            }
          }
        }
      }
      if items.is_empty() && after.is_empty() {
        None
      } else {
        Some((items, after))
      }
    } else {
      None
    }
  }
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditListingsData {
  children: Vec<RedditListingItem>
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditListingItem {
  kind: String,
  data: Option<RedditListingItemData>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditListingItemData {
  media_metadata: Option<HashMap<String, RedditGalleryItem>>,
  preview: Option<RedditPreview>,
  title: String,
  permalink: String,
  name: String,
  url: String,
  secure_media_embed: Option<RedditMediaEmbed>,
  media: Option<RedditMedia>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditMedia {
  #[serde(rename = "type")]
  type_: Option<String>,
  oembed: Option<OEmbed>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct OEmbed {
  thumbnail_url: Option<String>
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditMediaEmbed {
  scrolling: Option<bool>,
  width: Option<i32>,
  height: Option<i32>,
  content: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditPreview {
  images: Vec<RedditPreviewImageItem>
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditPreviewImageItem {
  source: RedditPreviewImage,
  resolutions: Vec<RedditPreviewImage>,
  variants: Option<RedditPreviewVariantItem>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditPreviewImageItemNoVars {
  source: RedditPreviewImage,
  resolutions: Vec<RedditPreviewImage>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditPreviewVariantItem {
  gif: Option<RedditPreviewImageItemNoVars>,
  mp4: Option<RedditPreviewImageItemNoVars>,
}

impl RedditPreviewImageItemNoVars {
  pub(crate) fn get_srcset(&self) -> String {
    let mut sizes = Vec::new();
    for image in &self.resolutions {
      sizes.push(image.to_srcset_value());
    }
    sizes.push(self.source.to_srcset_value());
    sizes.join(", ")
  }
}

impl RedditPreviewImageItem {
  pub(crate) fn get_srcset(&self) -> String {
    let mut sizes = Vec::new();
    for image in &self.resolutions {
      sizes.push(image.to_srcset_value());
    }
    sizes.push(self.source.to_srcset_value());
    sizes.join(", ")
  }
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditPreviewImage {
  url: String,
  width: i32,
  height: i32,
}

impl RedditPreviewImage {
  pub(crate) fn to_srcset_value(&self) -> String {
    format!("{} {}w", self.url.replace("&amp;", "&"), self.width)
  }
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditGalleryItemSource {
  x: i32,
  y: i32,
  u: String,
}

impl RedditGalleryItemSource {
  pub(crate) fn to_srcset_value(&self) -> String {
    format!("{} {}w", self.u.replace("&amp;", "&"), self.x)
  }
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditGalleryItem {
  s: RedditGalleryItemSource,
  p: Vec<RedditGalleryItemSource>,
}

impl RedditGalleryItem {
  pub(crate) fn get_srcset(&self) -> String {
    let mut sizes = Vec::new();
    for image in &self.p {
      sizes.push(image.to_srcset_value());
    }
    sizes.push(self.s.to_srcset_value());
    sizes.join(", ")
  }
}