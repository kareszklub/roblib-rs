use super::Event;
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ConcreteType {
    #[cfg(feature = "roland")]
    TrackSensor(super::TrackSensor),
    #[cfg(feature = "roland")]
    UltraSensor(super::UltraSensor),

    #[cfg(feature = "gpio")]
    GpioPin(super::GpioPin),

    #[cfg(feature = "camloc")]
    CamlocConnect(super::CamlocConnect),
    #[cfg(feature = "camloc")]
    CamlocDisconnect(super::CamlocDisconnect),
    #[cfg(feature = "camloc")]
    CamlocPosition(super::CamlocPosition),
    #[cfg(feature = "camloc")]
    CamlocInfoUpdate(super::CamlocInfoUpdate),

    None,
}

#[derive(Clone, Debug)]
pub enum ConcreteValue {
    #[cfg(feature = "roland")]
    TrackSensor(<super::TrackSensor as Event>::Item),
    #[cfg(feature = "roland")]
    UltraSensor(<super::UltraSensor as Event>::Item),

    #[cfg(feature = "gpio")]
    GpioPin(<super::GpioPin as Event>::Item),

    #[cfg(feature = "camloc")]
    CamlocConnect(<super::CamlocConnect as Event>::Item),
    #[cfg(feature = "camloc")]
    CamlocDisconnect(<super::CamlocDisconnect as Event>::Item),
    #[cfg(feature = "camloc")]
    CamlocPosition(<super::CamlocPosition as Event>::Item),
    #[cfg(feature = "camloc")]
    CamlocInfoUpdate(<super::CamlocInfoUpdate as Event>::Item),

    None,
}

impl Serialize for ConcreteType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ConcreteType", 2)?;
        match self {
            #[cfg(feature = "roland")]
            Self::TrackSensor(c) => {
                s.serialize_field("name", super::TrackSensor::NAME)?;
                s.serialize_field("ev", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::UltraSensor(c) => {
                s.serialize_field("name", super::UltraSensor::NAME)?;
                s.serialize_field("ev", &c)?;
            }

            #[cfg(feature = "gpio")]
            Self::GpioPin(c) => {
                s.serialize_field("name", super::GpioPin::NAME)?;
                s.serialize_field("ev", &c)?;
            }

            #[cfg(feature = "camloc")]
            Self::CamlocConnect(c) => {
                s.serialize_field("name", super::CamlocConnect::NAME)?;
                s.serialize_field("ev", &c)?;
            }
            #[cfg(feature = "camloc")]
            Self::CamlocDisconnect(c) => {
                s.serialize_field("name", super::CamlocDisconnect::NAME)?;
                s.serialize_field("ev", &c)?;
            }
            #[cfg(feature = "camloc")]
            Self::CamlocPosition(c) => {
                s.serialize_field("name", super::CamlocPosition::NAME)?;
                s.serialize_field("ev", &c)?;
            }
            #[cfg(feature = "camloc")]
            Self::CamlocInfoUpdate(c) => {
                s.serialize_field("name", super::CamlocInfoUpdate::NAME)?;
                s.serialize_field("ev", &c)?;
            }

            Self::None => unreachable!(),
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for ConcreteType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ConcreteTypeVisitor;
        impl<'de> Visitor<'de> for ConcreteTypeVisitor {
            type Value = ConcreteType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an event name and arguments")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // TODO: &str should be fine...
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                match &name[..] {
                    #[cfg(feature = "roland")]
                    super::TrackSensor::NAME => seq.next_element()?.map(ConcreteType::TrackSensor),
                    #[cfg(feature = "roland")]
                    super::UltraSensor::NAME => seq.next_element()?.map(ConcreteType::UltraSensor),

                    #[cfg(feature = "gpio")]
                    super::GpioPin::NAME => seq.next_element()?.map(ConcreteType::GpioPin),

                    #[cfg(feature = "camloc")]
                    super::CamlocConnect::NAME => {
                        seq.next_element()?.map(ConcreteType::CamlocConnect)
                    }
                    #[cfg(feature = "camloc")]
                    super::CamlocDisconnect::NAME => {
                        seq.next_element()?.map(ConcreteType::CamlocDisconnect)
                    }
                    #[cfg(feature = "camloc")]
                    super::CamlocPosition::NAME => {
                        seq.next_element()?.map(ConcreteType::CamlocPosition)
                    }
                    #[cfg(feature = "camloc")]
                    super::CamlocInfoUpdate::NAME => {
                        seq.next_element()?.map(ConcreteType::CamlocInfoUpdate)
                    }

                    _ => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(&name),
                            &"an event name",
                        ))
                    }
                }
                .ok_or_else(|| de::Error::invalid_length(0, &self))
            }
        }

        deserializer.deserialize_struct("ConcreteType", &["name", "ev"], ConcreteTypeVisitor)
    }
}
