use std::borrow::Cow;

use jiff::Timestamp;
use poem::web::Field;
use serde_json::Value;

use crate::{
    registry::{MetaSchema, MetaSchemaRef},
    types::{
        ParseError, ParseFromJSON, ParseFromMultipartField, ParseFromParameter, ParseResult,
        ToJSON, Type,
    },
};

impl Type for Timestamp {
    const IS_REQUIRED: bool = true;

    type RawValueType = Self;

    type RawElementValueType = Self;

    fn name() -> Cow<'static, str> {
        "string_date-time".into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema::new_with_format("string", "date-time")))
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.as_raw_value().into_iter())
    }
}

impl ParseFromJSON for Timestamp {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value = value.unwrap_or_default();
        if let Value::String(value) = value {
            Ok(value.parse()?)
        } else {
            Err(ParseError::expected_type(value))
        }
    }
}

impl ParseFromParameter for Timestamp {
    fn parse_from_parameter(value: &str) -> ParseResult<Self> {
        Ok(value.parse()?)
    }
}

impl ParseFromMultipartField for Timestamp {
    async fn parse_from_multipart(field: Option<Field>) -> ParseResult<Self> {
        match field {
            Some(field) => Ok(field.text().await?.parse()?),
            None => Err(ParseError::expected_input()),
        }
    }
}

impl ToJSON for Timestamp {
    fn to_json(&self) -> Option<Value> {
        Some(Value::String(self.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;
    use serde_json::json;

    use crate::types::{ParseFromJSON, ParseFromParameter, ToJSON};

    #[test]
    fn jiff_timestamp_from_json() {
        let ts_str = "2024-03-10T10:00:00Z";
        let json_value = json!(ts_str);
        let parsed_ts = Timestamp::parse_from_json(Some(json_value)).unwrap();
        let expected_ts: Timestamp = ts_str.parse().unwrap();
        assert_eq!(parsed_ts, expected_ts);
    }

    #[test]
    fn jiff_timestamp_to_json() {
        let ts_str = "2024-03-10T10:00:00Z";
        let ts: Timestamp = ts_str.parse().unwrap();
        let json_value = ts.to_json().unwrap();
        assert_eq!(json_value, json!(ts_str));
    }

    #[test]
    fn jiff_timestamp_from_parameter() {
        let ts_str = "2024-03-10T10:00:00Z";
        let parsed_ts = Timestamp::parse_from_parameter(ts_str).unwrap();
        let expected_ts: Timestamp = ts_str.parse().unwrap();
        assert_eq!(parsed_ts, expected_ts);
    }

    #[test]
    fn jiff_timestamp_from_json_invalid() {
        let json_value = json!(12345); // Invalid type
        assert!(Timestamp::parse_from_json(Some(json_value)).is_err());
    }

    #[test]
    fn jiff_timestamp_from_parameter_invalid() {
        assert!(Timestamp::parse_from_parameter("invalid-timestamp").is_err());
    }
}
