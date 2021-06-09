//! UsbMuxd protocol
//! @see https://github.com/libimobiledevice/libusbmuxd/blob/master/include/usbmuxd-proto.h
//! 
//#[macro_use]
use serde::{Serialize, Deserialize};

#[cfg(any(target_os = "linux", target_os = "mac"))]
pub const USBMUXD_SOCKET_NAME: &str = "/var/run/usbmuxd";
#[cfg(target_os = "windows")]
pub const USBMUXD_SOCKET_NAME: &str = "127.0.0.1:27015";

pub const PLIST_LIBUSBMUX_VERSION: u32 = 3;
pub const USBMUXD_PROTOCOL_VERSION: u32 = 1;

pub enum UsbmuxdResult {
    Ok = 0,
    BadCommand = 1,
    BadDev = 2,
    Connrefused = 3,
    // ???
    // ???
    BadVersion = 6,
}

pub enum UsbmuxdMsgType {
    Result = 1,
    Connect = 2,
    Listen = 3,
    DeviceAdd = 4,
    DeviceRemove = 5,
    DevicePaired = 6,
    //???
    Plist = 8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UsbmuxdHeader {
    pub length: u32,
    pub version: u32,
    pub message: u32,
    pub tag: u32,
}

#[derive(Debug)]
pub struct UsbmuxdResultMsg {
    pub header: UsbmuxdHeader,
    pub result: u32,
}

#[derive(Debug)]
pub struct UsbmuxdConnectRequest {
    pub header: UsbmuxdHeader,
    pub device_id: u32,
    pub port: u16,     // TCP port number
    pub reserved: u16, // set to zero
}

#[derive(Debug)]
pub struct UsbmuxdListenRequest {
    pub header: UsbmuxdHeader,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UsbmuxdDeviceProperties {
    pub connection_type: String,
    #[serde(rename = "DeviceID")]
    pub device_id: u32,
    #[serde(rename = "LocationID")]
    pub location_id: u32,
    #[serde(rename = "ProductID")]
    pub product_id: u32,
    pub serial_number: String,
    #[serde(rename = "UDID")]
    pub udid: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UsbmuxdDevice {
    #[serde(rename = "DeviceID")]
    pub device_id: u32,
    pub message_type: String,
    pub properties: UsbmuxdDeviceProperties,
}

/*
{
    "DeviceList": 
        Array([
                Dictionary(
                    {
                        "DeviceID": Integer(3),
                        "MessageType": String("Attached"),
                        "Properties": Dictionary({
                            "ConnectionType": String("USB"),
                            "DeviceID": Integer(3),
                            "LocationID": Integer(0),
                            "ProductID": Integer(4776),
                            "SerialNumber": String("97006ebdc8bc5daed2e354f4addae4fd2a81c52d"),
                            "UDID": String("97006ebdc8bc5daed2e354f4addae4fd2a81c52d")})
                    }
                )
        ])
}
*/
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UsbmuxdDeviceList {
    pub device_list: Vec<UsbmuxdDevice>,
}

/*
impl Iterator for UsbmuxdDeviceList {
    type Item = UsbmuxdDevice;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.device_list.get(self.curr);
    }
}
*/