use std::collections::HashMap;
use serde::Deserialize;


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

//TODO: imgur... gifv

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

          if let Some(media_embed) = &child_data.secure_media_embed {
            if let (Some(media_domain_url), Some(scrolling), Some(width), Some(height))
            = (&media_embed.media_domain_url, media_embed.scrolling, media_embed.width, media_embed.height) {
              items.push(RedditItem {
                title: child_data.title.clone(),
                title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                item: RedditItemType::Embed {
                  url: media_domain_url.replace("&amp;", "&"),
                  scrolling: if scrolling { "yes".to_string() } else { "no".to_string() },
                  width,
                  height,
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
  secure_media_embed: Option<RedditMediaEmbed>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditMediaEmbed {
  media_domain_url: Option<String>,
  scrolling: Option<bool>,
  width: Option<i32>,
  height: Option<i32>,
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