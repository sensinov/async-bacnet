mod io;
mod client;
mod error;
pub mod discover;

pub use client::Client;
pub use error::Error;
pub use io::TokioUdpIo;

// Re-export commonly used embedded-bacnet types
pub use embedded_bacnet::application_protocol::primitives::data_value::{
    ApplicationDataValue, ApplicationDataValueWrite, Enumerated,
};
pub use embedded_bacnet::application_protocol::services::read_property::{
    ReadProperty, ReadPropertyAck, ReadPropertyValue,
};
pub use embedded_bacnet::application_protocol::services::read_property_multiple::{
    ReadPropertyMultiple, ReadPropertyMultipleAck,
};
pub use embedded_bacnet::application_protocol::services::write_property::WriteProperty;
pub use embedded_bacnet::common::object_id::{ObjectId, ObjectType};
pub use embedded_bacnet::common::property_id::PropertyId;
pub use embedded_bacnet::common::spec::Binary;
