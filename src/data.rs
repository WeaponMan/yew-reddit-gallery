use std::collections::HashMap;
use serde::Deserialize;

pub(crate) struct RedditPicture {
  pub(crate) url: String,
  pub(crate) title: String,
  pub(crate) source_set: String,
  pub(crate) title_url: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RedditListings {
  data: Option<RedditListingsData>,
}

impl RedditListings {
  pub(crate) fn get_images(self) -> Option<(Vec<RedditPicture>, String)> {
    if let Some(data) = self.data {
      let mut images = Vec::new();
      let mut after = String::new();
      for child in data.children {
        if let Some(child_data) = child.data {
          after = child_data.name;
          if child.kind != "t3" {
            continue;
          }
          if let Some(media_metadata) = child_data.media_metadata {
            if !media_metadata.is_empty() {
              for item in media_metadata {
                images.push(RedditPicture {
                  url: item.1.s.u.replace("&amp;", "&"),
                  title: child_data.title.clone(),
                  source_set: item.1.get_srcset(),
                  title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                });
              }
              continue;
            }
          }
          if let Some(preview) = child_data.preview {
            if !preview.images.is_empty() {
              for item in preview.images {
                images.push(RedditPicture {
                  url: item.source.url.replace("&amp;", "&"),
                  title: child_data.title.clone(),
                  source_set: item.get_srcset(),
                  title_url: format!("https://www.reddit.com/{}", &child_data.permalink),
                });
              }
              continue;
            }
          }
        }
      }
      if images.is_empty() && after.is_empty() {
        None
      } else {
        Some((images, after))
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
    sizes.push(format!("{}", self.s.to_srcset_value()));
    sizes.join(", ")
  }
}