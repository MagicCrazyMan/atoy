// use std::marker::PhantomData;

// use async_trait::async_trait;
// use url::Url;
// use wasm_bindgen::{JsCast, JsValue};
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestInit, Response};

// use crate::{
//     core::web::webgl::buffer::{BufferData, BufferSourceAsync},
//     window,
// };

// /// A loader loads resources from network using [`fetch`].
// pub struct FetchLoader<ResponseKind> {
//     url: Url,

//     _kind: PhantomData<ResponseKind>,
// }

// impl<ResponseKind> FetchLoader<ResponseKind> {
//     pub async fn load(&mut self) -> Result<(), JsValue> {
//         let init = RequestInit::new();
//         let request = Request::new_with_str_and_init(self.url.as_str(), &init)?;
//         let resp = JsFuture::from(window().fetch_with_request(&request)).await?;

//         let resp = resp.dyn_into::<Response>()?;

//         Ok(())
//     }
// }

// #[async_trait]
// impl BufferSourceAsync for FetchLoader<ResponseAsArrayBuffer> {
//     async fn load(&mut self) -> Result<BufferData, String> {
//         todo!()
//     }
// }

// #[async_trait]
// pub trait ResponseAs {
//     type Data;

//     fn new() -> Self;

//     async fn from_response(resp: Response) -> Result<Self::Data, JsValue>;
// }

// pub struct ResponseAsText;

// #[async_trait]
// impl ResponseAs for ResponseAsText {
//     type Data = String;

//     fn new() -> Self {
//         Self
//     }

//     async fn from_response(resp: Response) -> Result<Self::Data, JsValue> {
//         let text = JsFuture::from(resp.text()?).await?;
//         let text = text.as_string().unwrap_or("".to_string());
//         Ok(text)
//     }
// }

// pub struct ResponseAsArrayBuffer {}

// pub struct ResponseAsBlob {}

// pub struct ResponseAsJson {}

// pub struct ResponseAsFormData {}
