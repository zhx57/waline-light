use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  entities::wl_comment,
  helpers::{avatar::get_avatar, ip::Ip2Region, markdown::render_md_to_html, ua},
};

#[derive(Serialize, Clone)]
pub struct DataEntry {
  pub status: String,
  pub like: Option<i32>,
  pub link: Option<String>,
  pub mail: Option<String>,
  pub nick: Option<String>,
  pub user_id: Option<i32>,
  pub browser: String,
  pub os: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "objectId")]
  pub object_id: u32,
  pub ip: Option<String>,
  pub orig: Option<String>,
  pub url: Option<String>,
  pub pid: Option<i32>,
  pub rid: Option<i32>,
  pub time: i64,
  pub comment: Option<String>,
  pub avatar: String,
  pub level: Option<usize>,
  pub label: Option<String>,
  pub sticky: Option<i8>,
  pub addr: Option<String>,
  pub children: Vec<DataEntry>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reply_user: Option<Value>,
}

fn is_strictly_increasing(nums: &[usize]) -> bool {
  nums.windows(2).all(|w| w[0] < w[1])
}

fn get_thresholds(levels: &str) -> Vec<usize> {
  let mut thresholds = vec![];
  for s in levels.split(',') {
    if s.chars().all(|c| c.is_ascii_digit()) {
      thresholds.push(s.parse().unwrap());
    } else {
      return vec![0, 10, 20, 50, 100, 200];
    }
  }
  if !is_strictly_increasing(&thresholds) {
    return vec![0, 10, 20, 50, 100, 200];
  }
  thresholds
}

pub fn get_level(count: usize, levels: &str) -> usize {
  let thresholds = get_thresholds(levels);
  for (index, &threshold) in thresholds.iter().enumerate() {
    if count < threshold {
      return index - 1;
    }
  }
  0
}

pub fn build_data_entry(
  comment: wl_comment::Model,
  level: Option<usize>,
  ip2region: &Option<Ip2Region>,
  disable_useragent: bool,
  disable_region: bool,
) -> DataEntry {
  let (browser, os) = if disable_useragent {
    ("".to_string(), "".to_string())
  } else {
    ua::parse(comment.ua.unwrap_or("".to_owned()))
  };
  let addr = if disable_region {
    Some("".to_string())
  } else if let Some(ip) = comment.ip {
    if let Some(ip2region) = ip2region {
      ip2region.search(&ip)
    } else {
      None
    }
  } else {
    None
  };
  let safe_html = if let Some(ref comment_text) = comment.comment {
    Some(ammonia::clean(&render_md_to_html(comment_text)))
  } else {
    Some("".to_string())
  };
  DataEntry {
    status: comment.status,
    like: comment.like,
    link: comment.link,
    mail: None,
    nick: comment.nick,
    user_id: comment.user_id,
    browser,
    os,
    url: comment.url,
    r#type: None,
    object_id: comment.id,
    ip: None,
    orig: comment.comment,
    time: comment.created_at.unwrap().timestamp_millis(),
    pid: comment.pid,
    rid: comment.rid,
    comment: safe_html,
    avatar: get_avatar(&comment.mail.clone().unwrap_or("default".to_owned())),
    level,
    label: None,
    sticky: comment.sticky,
    addr,
    children: vec![],
    reply_user: None,
  }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCommentQuery {
  pub lang: String,
  pub path: Option<String>,
  pub page_size: Option<i32>,
  pub page: i32,
  pub sort_by: Option<String>,
  pub r#type: Option<String>,
  pub owner: Option<String>,
  pub status: Option<String>,
  pub keyword: Option<String>,
}

impl GetCommentQuery {
  pub fn validate_by_path(&self) -> Result<(), Vec<&'static str>> {
    let mut missing_fields = Vec::new();
    // 检查 Option 字段
    if self.path.is_none() {
      missing_fields.push("path");
    }
    if self.page_size.is_none() {
      missing_fields.push("pageSize");
    }
    if self.sort_by.is_none() {
      missing_fields.push("sortBy");
    }
    if missing_fields.is_empty() {
      Ok(())
    } else {
      Err(missing_fields)
    }
  }
  pub fn validate_by_admin(&self) -> Result<(), Vec<&'static str>> {
    let mut missing_fields = Vec::new();
    if self.r#type.is_none() {
      missing_fields.push("type");
    }
    if self.owner.is_none() {
      missing_fields.push("owner");
    }
    if self.status.is_none() {
      missing_fields.push("status");
    }
    if self.keyword.is_none() {
      missing_fields.push("keyword");
    }
    if missing_fields.is_empty() {
      Ok(())
    } else {
      Err(missing_fields)
    }
  }
}

pub fn create_comment_model(
  user_id: Option<i32>,
  comment: String,
  link: Option<String>,
  mail: String,
  nick: String,
  ua: String,
  url: String,
  ip: String,
  pid: Option<i32>,
  rid: Option<i32>,
) -> wl_comment::ActiveModel {
  let utc_time = helpers::time::utc_now();
  wl_comment::ActiveModel {
    user_id: Set(user_id),
    comment: Set(Some(comment)),
    link: Set(link),
    mail: Set(Some(mail)),
    nick: Set(Some(nick)),
    ua: Set(Some(ua)),
    url: Set(Some(url)),
    status: Set("approved".to_string()),
    pid: Set(pid),
    rid: Set(rid),
    inserted_at: Set(Some(utc_time)),
    created_at: Set(Some(utc_time)),
    updated_at: Set(Some(utc_time)),
    ip: Set(Some(ip)),
    ..Default::default()
  }
}

pub fn has_forbidden_word(comment: &str, forbidden_words: &Vec<String>) -> bool {
  for word in forbidden_words {
    if comment.contains(word) {
      return true;
    }
  }
  false
}

pub enum UserType {
  Anonymous,
  Guest(String),
  Administrator(String),
}

#[derive(Deserialize)]
pub struct CreateCommentQuery {
  pub lang: String,
}

#[derive(Deserialize, Clone)]
pub struct CreateCommentBody {
  pub comment: String,
  // or ""
  pub link: Option<String>,
  // or ""
  pub mail: String,
  // or ""
  pub nick: String,
  // user-agent
  pub ua: String,
  // path
  pub url: String,
  // Parent comment ID
  pub pid: Option<i32>,
  // span id
  pub rid: Option<i32>,
  // at
  pub at: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCommentBody {
  pub status: Option<String>,
  pub like: Option<bool>,
  pub comment: Option<String>,
  pub link: Option<String>,
  pub mail: Option<String>,
  pub nick: Option<String>,
  pub ua: Option<String>,
  pub url: Option<String>,
  pub sticky: Option<i8>,
}
