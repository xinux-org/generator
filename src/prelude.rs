#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct NixosPkgList {
    pub packages: HashMap<String, NixosPkg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NixosPkg {
    pub pname: Option<String>,
    pub version: Option<String>,
    pub system: String,
    pub meta: Meta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meta {
    pub broken: Option<bool>,
    pub insecure: Option<bool>,
    pub unsupported: Option<bool>,
    pub unfree: Option<bool>,
    pub description: Option<String>,
    #[serde(rename = "longDescription")]
    pub longdescription: Option<String>,
    pub homepage: Option<StrOrVec>,
    pub maintainers: Option<Value>,
    pub position: Option<String>,
    pub license: Option<LicenseEnum>,
    pub platforms: Option<Platform>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum StrOrVec {
    Single(String),
    List(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Platform {
    Single(String),
    List(Vec<String>),
    ListList(Vec<Vec<String>>),
    Unknown(Value),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LicenseEnum {
    Single(License),
    List(Vec<License>),
    SingleStr(String),
    VecStr(Vec<String>),
    Mixed(Vec<LicenseEnum>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct License {
    pub free: Option<bool>,
    #[serde(rename = "fullName")]
    pub fullname: Option<String>,
    #[serde(rename = "spdxId")]
    pub spdxid: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PkgMaintainer {
    pub email: Option<String>,
    pub github: Option<String>,
    pub matrix: Option<String>,
    pub name: Option<String>,
}
