#![forbid(unsafe_code)]

use crate::web_search_plan::url::UrlFetchErrorKind;
use flate2::read::{GzDecoder, ZlibDecoder};
use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncoding {
    Identity,
    Gzip,
    Brotli,
    Deflate,
}

pub fn parse_content_encoding(header: Option<&str>) -> Result<ContentEncoding, UrlFetchErrorKind> {
    let Some(raw) = header else {
        return Ok(ContentEncoding::Identity);
    };
    let token = raw
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match token.as_str() {
        "" | "identity" => Ok(ContentEncoding::Identity),
        "gzip" => Ok(ContentEncoding::Gzip),
        "br" => Ok(ContentEncoding::Brotli),
        "deflate" => Ok(ContentEncoding::Deflate),
        _ => Err(UrlFetchErrorKind::UnsupportedContentEncoding),
    }
}

pub fn wrap_decoder<R: Read + 'static>(
    reader: R,
    encoding: ContentEncoding,
) -> Box<dyn Read> {
    match encoding {
        ContentEncoding::Identity => Box::new(reader),
        ContentEncoding::Gzip => Box::new(GzDecoder::new(reader)),
        ContentEncoding::Brotli => Box::new(brotli::Decompressor::new(reader, 4096)),
        ContentEncoding::Deflate => Box::new(ZlibDecoder::new(reader)),
    }
}
