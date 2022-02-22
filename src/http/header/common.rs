use super::{Header, IntoHeader};
use crate::http::ByteStr;

/// HTTP Content-Types.
pub enum ContentType {
    Jpeg,
    Png,
    Json,
    Xml,
    Html,
    Text,
    FormData,
    FormUrlEncoded,
    OctetStream,
    Other(mime::Mime),
}

impl IntoHeader for ContentType {
    fn into_header(self) -> Header {
        let value = match self {
            ContentType::Jpeg => ByteStr::from_static("image/jpeg"),
            ContentType::Png => ByteStr::from_static("image/png"),
            ContentType::Json => ByteStr::from_static("application/json"),
            ContentType::Xml => ByteStr::from_static("text/xml"),
            ContentType::Html => ByteStr::from_static("text/html"),
            ContentType::Text => ByteStr::from_static("text/plain"),
            ContentType::FormData => ByteStr::from_static("multipart/form-data"),
            ContentType::FormUrlEncoded => {
                ByteStr::from_static("application/x-www-form-urlencoded")
            }
            ContentType::OctetStream => ByteStr::from_static("multipart"),
            ContentType::Other(mime) => ByteStr::from(mime.type_().as_str()),
        };

        Header {
            name: ByteStr::from_static("content-type"),
            value,
        }
    }
}
