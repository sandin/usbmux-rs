//! UsbMux-Rust
extern crate bincode;
extern crate plist;
extern crate serde;

mod error;
mod plist_utils;
mod protocol;

use std::io;
use std::result;
//use std::io::prelude::*;
use std::io::Cursor;
use std::io::{BufReader, BufWriter, Read, Write};
use std::mem;
use std::time::Duration;
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::vec::Vec;
use std::thread;
use std::sync::{Arc, Mutex, mpsc};

use plist::Value;
use plist_utils::{create_plist_message, plist_to_binary_data, plist_to_object};
use serde::de;

use error::{Error, ErrorKind, Result};

use protocol::{
    UsbmuxdDevice, UsbmuxdDeviceList, UsbmuxdHeader, UsbmuxdMsgType, PLIST_LIBUSBMUX_VERSION,
    USBMUXD_PROTOCOL_VERSION, USBMUXD_SOCKET_NAME,
};

pub trait UsbmuxdEventListener {
    fn on_event(&mut self, event: &String);
}

//type UsbmuxdEventListener = Box<dyn Fn() + Send + 'static>;

pub struct UsbmuxdClient
{
    listeners: Arc<Mutex<Vec<Box<dyn UsbmuxdEventListener>>>>,
    worker: Option<thread::JoinHandle<()>>,
    sender: Option<mpsc::Sender<()>>,
}

impl UsbmuxdClient {
    fn new() -> UsbmuxdClient {
        UsbmuxdClient {
            listeners: Arc::new(Mutex::new(Vec::new())),
            worker: Option::None,
            sender: Option::None
        }
    }

    fn connect_usbmuxd_socket(&mut self) -> Result<TcpStream> {
        let stream = TcpStream::connect(USBMUXD_SOCKET_NAME)?;
        Ok(stream)
    }

    fn disconnect_usbmuxd_socket(&mut self, stream: &TcpStream) -> Result<()> {
        stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    fn send_plist_packet(&mut self, stream: &mut TcpStream, value: &Value) -> Result<()> {
        let header_length: usize = mem::size_of::<UsbmuxdHeader>();
        let data: Vec<u8> = plist_to_binary_data(&value);
        let length: usize = header_length + data.len();

        let header = UsbmuxdHeader {
            length: length as u32,
            version: USBMUXD_PROTOCOL_VERSION,
            message: UsbmuxdMsgType::Plist as u32,
            tag: 1, // TODO
        };
        println!("request header: {:?}", header);
        let header: Vec<u8> = bincode::serialize(&header).unwrap();
        //println!("encoded: {:?} {:?}", to_hex_string(&header), header);

        stream.write_all(&header)?;
        stream.write_all(&data)?;

        Ok(())
    }

    fn recv_plist_packet<T: de::DeserializeOwned>(&mut self, stream: &mut TcpStream) -> Result<T> {
        let header_length: usize = mem::size_of::<UsbmuxdHeader>();
        let mut buffer = vec![0u8; header_length];
        stream.read_exact(&mut buffer)?;

        let header: UsbmuxdHeader = bincode::deserialize(&buffer[..])?;
        println!("response header: {:?}", header);

        let length: usize = header.length as usize - header_length;
        buffer = vec![0u8; length];
        stream.read_exact(&mut buffer)?;

        if header.message == UsbmuxdMsgType::Plist as u32 {
            let payload: T = plist_to_object(&buffer);
            //println!("response payload: {:?}", payload);
            Ok(payload)
        } else {
            Err(Box::new(ErrorKind::Custom(format!(
                "Unknown msg type: {}",
                header.message
            ))))
        }
    }

    /// Contacts usbmuxd and retrieves a list of connected devices.
    pub fn get_device_list(&mut self) -> Result<Vec<UsbmuxdDevice>> {
        let mut stream = match self.connect_usbmuxd_socket() {
            Ok(r) => r,
            Err(e) => return Err(Box::new(ErrorKind::Connection())),
        };

        let plist_msg = create_plist_message(String::from("ListDevices"));
        match self.send_plist_packet(&mut stream, &plist_msg) {
            Ok(r) => r,
            Err(e) => return Err(Box::new(ErrorKind::Connection())),
        };
        let response: UsbmuxdDeviceList = self.recv_plist_packet(&mut stream)?;

        Ok(response.device_list)
    }

    pub fn start_listen(&mut self) {
        let (sender, receiver) = mpsc::channel();
        self.sender = Some(sender);

        let listeners = self.listeners.clone();
        self.worker = Some(thread::spawn(move || {
            println!("thread start, tid={:?}!", thread::current().id());
            loop {
                if receiver.try_recv().is_ok() {
                    println!("stop the thread");
                    break;
                }

                for listener in &*listeners.lock().unwrap() {
                    listener.on_event(&String::from(""));
                }

                thread::sleep_ms(100);
            }
            println!("thread exit, tid={:?}!", thread::current().id());
        }));
    }

    pub fn stop_listen(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.send(()).unwrap();
        }
        if let Some(thread_handle) = self.worker.take() {
            thread_handle.join().unwrap();
        }
    }

    /// Subscribe a callback function to be called upon device add/remove events.
    pub fn events_subscribe(&mut self, listener: Box<dyn UsbmuxdEventListener>)
    {
        self.listeners.lock().unwrap().push(listener);

        if self.worker.is_none() {
            self.start_listen();
        }
    }

    /// Subscribe a callback function to be called upon device add/remove events.
    pub fn events_unsubscribe(&mut self, listener: Box<dyn UsbmuxdEventListener>) 
    {
        let listeners : &Vec<Box<dyn UsbmuxdEventListener>> = &*self.listeners.lock().unwrap();
        let index = listeners.iter().position(|item| { 
            return **item == listener;
        }).unwrap();
        listeners.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn to_hex_string(bytes: &Vec<u8>) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
        strs.connect(" ")
    }

    #[test]
    fn get_devices_list() -> result::Result<(), Box<Error>> {
        let mut client = create_client()?;

        let callback = || {
            println!("on event listener callback");
        };
        client.events_subscribe(callback);

        client.start_listen();
        thread::sleep(Duration::from_secs(3));

        client.events_unsubscribe(callback);
        thread::sleep(Duration::from_secs(3));

        client.stop_listen();

        //let listener = create_listener();
        //client.events_subscribe(&listener);
        //client.events_unsubscribe(&listener);

        Ok(())
    }

    fn create_client() -> result::Result<UsbmuxdClient, Box<Error>> {
        let mut client = UsbmuxdClient::new();

        let devices = client.get_device_list()?;
        println!("devices: {:?}", devices);
        assert_ne!(devices.len(), 0);

        for device in devices {
            println!("device: {:?}", device);
            let udid = device.properties.udid;
            assert!(udid.len() > 0);
        }

        Ok(client)
    }

}
