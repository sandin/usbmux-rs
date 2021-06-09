use std::io::Cursor;

use plist::dictionary::Dictionary;
use plist::Integer;
use plist::Value;

use serde::de;

use crate::protocol::PLIST_LIBUSBMUX_VERSION;

pub fn plist_to_xml_data(value: &Value) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    value.to_writer_xml(&mut buf).unwrap();
    buf.into_inner()
}

pub fn plist_to_binary_data(value: &Value) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    value.to_writer_binary(&mut buf).unwrap();
    buf.into_inner()
}

pub fn plist_to_object<T: de::DeserializeOwned>(data: &Vec<u8>) -> T {
    let mut buf = Cursor::new(data.as_slice());
    plist::from_reader(&mut buf).unwrap()
}

pub fn create_plist_message(message_type: String) -> Value {
    let client_name: String = env!("CARGO_PKG_NAME").to_owned();
    let client_version: String = env!("CARGO_PKG_VERSION").to_owned();

    let mut dict = Dictionary::new();
    dict.insert("MessageType".to_owned(), Value::String(message_type));
    dict.insert(
        "ClientVersionString".to_owned(),
        Value::String(client_version),
    );
    dict.insert("ProgName".to_owned(), Value::String(client_name));
    dict.insert(
        "kLibUSBMuxVersion".to_owned(),
        Value::Integer(Integer::from(PLIST_LIBUSBMUX_VERSION)),
    );

    Value::Dictionary(dict)
}
