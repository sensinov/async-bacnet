use clap::{Args, Parser};
use eyre::{eyre, Result};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use async_bacnet::{
    ApplicationDataValue, ApplicationDataValueWrite, Binary, Client, Enumerated, ObjectId,
    ObjectType, PropertyId, ReadProperty, WriteProperty,
};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ApplicationDataValueArg {
    Boolean,
    Real,
    Enumerated,
    EnumeratedBinary,
}

impl ApplicationDataValueArg {
    fn to_value(&self, value: serde_json::Value) -> ApplicationDataValueWrite<'static> {
        match &value {
            serde_json::Value::Bool(v) => match self {
                ApplicationDataValueArg::Boolean => ApplicationDataValueWrite::Boolean(*v),
                ApplicationDataValueArg::EnumeratedBinary => {
                    ApplicationDataValueWrite::Enumerated(Enumerated::Binary(if *v {
                        Binary::On
                    } else {
                        Binary::Off
                    }))
                }
                _ => panic!("invalid value type"),
            },
            serde_json::Value::Number(v) => match self {
                ApplicationDataValueArg::Real => {
                    ApplicationDataValueWrite::Real(v.as_f64().expect("invalid number") as f32)
                }
                _ => panic!("invalid value {value:?} for type {self:?}"),
            },
            _ => panic!("invalid value {value:?}"),
        }
    }
}

#[derive(Debug, Args, Clone)]
struct WriteArgs {
    #[clap(short, long, requires = "write_value", required = false)]
    write_value: String,
    #[clap(short = 't', long, required = false)]
    write_type: ApplicationDataValueArg,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[repr(u32)]
pub enum ArgObjectType {
    ObjectAnalogInput = 0,
    ObjectAnalogOutput = 1,
    ObjectAnalogValue = 2,
    ObjectBinaryInput = 3,
    ObjectBinaryOutput = 4,
    ObjectBinaryValue = 5,
    ObjectCalendar = 6,
    ObjectCommand = 7,
    ObjectDevice = 8,
    ObjectEventEnrollment = 9,
    ObjectFile = 10,
    ObjectGroup = 11,
    ObjectLoop = 12,
    ObjectMultiStateInput = 13,
    ObjectMultiStateOutput = 14,
    ObjectNotificationClass = 15,
    ObjectProgram = 16,
    ObjectSchedule = 17,
    ObjectAveraging = 18,
    ObjectMultiStateValue = 19,
    ObjectTrendlog = 20,
    ObjectLifeSafetyPoint = 21,
    ObjectLifeSafetyZone = 22,
    ObjectAccumulator = 23,
    ObjectPulseConverter = 24,
    ObjectEventLog = 25,
    ObjectGlobalGroup = 26,
    ObjectTrendLogMultiple = 27,
    ObjectLoadControl = 28,
    ObjectStructuredView = 29,
    ObjectAccessDoor = 30,
    ObjectTimer = 31,
    ObjectAccessCredential = 32,
    ObjectAccessPoint = 33,
    ObjectAccessRights = 34,
    ObjectAccessUser = 35,
    ObjectAccessZone = 36,
    ObjectCredentialDataInput = 37,
    ObjectNetworkSecurity = 38,
    ObjectBitstringValue = 39,
    ObjectCharacterstringValue = 40,
    ObjectDatePatternValue = 41,
    ObjectDateValue = 42,
    ObjectDatetimePatternValue = 43,
    ObjectDatetimeValue = 44,
    ObjectIntegerValue = 45,
    ObjectLargeAnalogValue = 46,
    ObjectOctetstringValue = 47,
    ObjectPositiveIntegerValue = 48,
    ObjectTimePatternValue = 49,
    ObjectTimeValue = 50,
    ObjectNotificationForwarder = 51,
    ObjectAlertEnrollment = 52,
    ObjectChannel = 53,
    ObjectLightingOutput = 54,
    ObjectBinaryLightingOutput = 55,
    ObjectNetworkPort = 56,
    Reserved = 57,
    Proprietary = 128,
    Invalid = 1024,
}

#[derive(Debug, Parser, Clone)]
#[command(version)]
struct BacnetCliArgs {
    url: SocketAddr,
    object_type: ArgObjectType,
    instance: u32,
    #[clap(short, long, default_value = "85")]
    property: u32,

    #[clap(flatten)]
    write: Option<WriteArgs>,
}

impl BacnetCliArgs {
    fn object_id(&self) -> Result<ObjectId> {
        let object_type: ObjectType = (self.object_type as u32)
            .try_into()
            .map_err(|e| eyre!("invalid object type: {e}"))?;
        Ok(ObjectId::new(object_type, self.instance))
    }

    fn property_id(&self) -> Result<PropertyId> {
        self.property
            .try_into()
            .map_err(|e| eyre!("invalid object property: {e}"))
    }

    fn write_value(&self) -> Option<ApplicationDataValueWrite<'static>> {
        self.write.as_ref().map(|v| {
            v.write_type
                .to_value(serde_json::from_str(&v.write_value).expect("invalid json"))
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let default_log_level = "bacnet_cli=info".parse().unwrap();
    Registry::default()
        .with(tracing_subscriber::fmt::Layer::default())
        .with(
            EnvFilter::builder()
                .with_default_directive(default_log_level)
                .from_env_lossy(),
        )
        .init();

    let args = BacnetCliArgs::parse();
    let object_id = args.object_id()?;
    let property_id = args.property_id()?;

    let mut client = Client::new(args.url)
        .await
        .map_err(|e| eyre!("failed to create client: {e:?}"))?;

    if let Some(write_value) = args.write_value() {
        let request = WriteProperty::new(object_id, property_id, None, None, write_value);
        client
            .write_property(request)
            .await
            .map_err(|e| eyre!("failed to write property: {e:?}"))?;
        println!("write done");
    } else {
        let request = ReadProperty::new(object_id, property_id);
        let ack = client
            .read_property(request)
            .await
            .map_err(|e| eyre!("failed to read property: {e:?}"))?;
        let value: ApplicationDataValue = ack
            .property_value
            .try_into()
            .map_err(|e| eyre!("failed to parse property value: {e:?}"))?;
        println!("{value:?}");
    }

    Ok(())
}
