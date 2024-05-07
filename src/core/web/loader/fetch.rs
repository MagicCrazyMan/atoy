use std::{fmt::Display, marker::PhantomData};

use async_trait::async_trait;
use js_sys::{ArrayBuffer, Object};
use serde::de::DeserializeOwned;
use url::Url;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, FormData, Request, RequestInit, Response};

use crate::{
    core::web::webgl::buffer::{BufferData, BufferSourceAsync},
    window,
};

/// A loader loads resources from network using [`fetch`].
pub struct FetchLoader<ReadAs = ()> {
    url: Url,
    _kind: PhantomData<ReadAs>,
}

impl<ReadAs> FetchLoader<ReadAs> {
    pub async fn send(&self) -> Result<Response, Error> {
        let init = RequestInit::new();
        let request = Request::new_with_str_and_init(self.url.as_str(), &init)?;
        let resp = JsFuture::from(window().fetch_with_request(&request)).await?;
        let resp = resp.dyn_into::<Response>()?;
        Ok(resp)
    }

    pub async fn load_as_text(&self) -> Result<String, Error> {
        let resp = self.send().await?;
        let text = JsFuture::from(resp.text()?)
            .await?
            .as_string()
            .unwrap_or(String::new());
        Ok(text)
    }

    pub async fn load_as_json(&self) -> Result<Object, Error> {
        let resp = self.send().await?;
        let json = JsFuture::from(resp.json()?).await?.dyn_into::<Object>()?;
        Ok(json)
    }

    pub async fn load_as_form_data(&self) -> Result<FormData, Error> {
        let resp = self.send().await?;
        let form_data = JsFuture::from(resp.form_data()?)
            .await?
            .dyn_into::<FormData>()?;
        Ok(form_data)
    }

    pub async fn load_as_blob(&self) -> Result<Blob, Error> {
        let resp = self.send().await?;
        let blob = JsFuture::from(resp.blob()?).await?.dyn_into::<Blob>()?;
        Ok(blob)
    }

    pub async fn load_as_array_buffer(&self) -> Result<ArrayBuffer, Error> {
        let resp = self.send().await?;
        let array_buffer = JsFuture::from(resp.array_buffer()?)
            .await?
            .dyn_into::<ArrayBuffer>()?;
        Ok(array_buffer)
    }
}

impl FetchLoader<AsText> {
    pub async fn load(&self) -> Result<String, Error> {
        self.load_as_text().await
    }
}

impl FetchLoader<AsJson> {
    pub async fn load(&self) -> Result<Object, Error> {
        self.load_as_json().await
    }
}

impl FetchLoader<AsJsonDeserialize> {
    pub async fn load<T>(&self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let text = self.load_as_text().await?;
        let value = serde_json::from_value::<T>(serde_json::Value::String(text))?;
        Ok(value)
    }
}

impl FetchLoader<AsFormData> {
    pub async fn load(&self) -> Result<FormData, Error> {
        self.load_as_form_data().await
    }
}

impl FetchLoader<AsBlob> {
    pub async fn load(self) -> Result<Blob, Error> {
        self.load_as_blob().await
    }
}

impl FetchLoader<AsArrayBuffer> {
    pub async fn load(&self) -> Result<ArrayBuffer, Error> {
        self.load_as_array_buffer().await
    }
}

#[async_trait(?Send)]
impl BufferSourceAsync for FetchLoader<AsArrayBuffer> {
    async fn load(&mut self) -> Result<BufferData, String> {
        let data = Self::load(&self).await.map_err(|err| err.to_string())?;
        Ok(BufferData::ArrayBuffer { data })
    }
}

pub struct AsText;

pub struct AsJson;

pub struct AsJsonDeserialize;

pub struct AsFormData;

pub struct AsBlob;

pub struct AsArrayBuffer;

#[derive(Debug)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        todo!()
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(value: serde_json::error::Error) -> Self {
        todo!()
    }
}
