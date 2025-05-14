use std::borrow::Cow;

use ndarray::Array2;
use serde_json::Value;

use crate::{
    registry::{MetaSchema, MetaSchemaRef, Registry},
    types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type},
};

impl<T: Type> Type for Array2<T> {
    const IS_REQUIRED: bool = true;

    type RawValueType = Self;

    type RawElementValueType = T::RawValueType;

    fn name() -> Cow<'static, str> {
        format!("ndarray2_{}", T::name()).into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema {
            items: Some(Box::new(MetaSchemaRef::Inline(Box::new(MetaSchema {
                items: Some(Box::new(T::schema_ref())),
                ..MetaSchema::new("array")
            })))),
            ..MetaSchema::new("array")
        }))
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.iter().filter_map(|item| item.as_raw_value()))
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T: ParseFromJSON> ParseFromJSON for Array2<T> {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value = value.unwrap_or_default();
        match value {
            Value::Array(rows) => {
                if rows.is_empty() {
                    return Ok(Array2::from_shape_vec((0, 0), vec![]).expect("valid shape"));
                }

                let first_row = match &rows[0] {
                    Value::Array(cols) => cols,
                    _ => return Err(ParseError::custom("Expected array of arrays")),
                };
                let n_rows = rows.len();
                let n_cols = first_row.len();

                // Validate all rows have same length
                for row in &rows {
                    match row {
                        Value::Array(cols) if cols.len() == n_cols => {}
                        _ => return Err(ParseError::custom("All rows must have same length")),
                    }
                }

                let mut data = Vec::with_capacity(n_rows * n_cols);
                for row in rows {
                    match row {
                        Value::Array(cols) => {
                            for col in cols {
                                let value =
                                    T::parse_from_json(Some(col)).map_err(ParseError::propagate)?;
                                data.push(value);
                            }
                        }
                        _ => unreachable!(),
                    }
                }

                Ok(Array2::from_shape_vec((n_rows, n_cols), data)
                    .map_err(|e| ParseError::custom(e.to_string()))?)
            }
            _ => Err(ParseError::expected_type(value)),
        }
    }
}

impl<T: ToJSON> ToJSON for Array2<T> {
    fn to_json(&self) -> Option<Value> {
        let shape = self.shape();
        let mut rows = Vec::with_capacity(shape[0]);

        for row_idx in 0..shape[0] {
            let mut row = Vec::with_capacity(shape[1]);
            for col_idx in 0..shape[1] {
                if let Some(value) = self[[row_idx, col_idx]].to_json() {
                    row.push(value);
                }
            }
            rows.push(Value::Array(row));
        }

        Some(Value::Array(rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn empty_array2() {
        let json = serde_json::json!([]);
        let arr = Array2::<i32>::parse_from_json(Some(json)).unwrap();
        assert_eq!(arr.shape(), &[0, 0]);
    }

    #[test]
    fn parse_array2() {
        let json = serde_json::json!([[1, 2, 3], [4, 5, 6]]);
        let arr = Array2::<f64>::parse_from_json(Some(json)).unwrap();
        assert_eq!(arr, arr2(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]));
    }

    #[test]
    fn array2_to_json() {
        let arr = arr2(&[[1, 2, 3], [4, 5, 6]]);
        let json = arr.to_json().unwrap();
        assert_eq!(json, serde_json::json!([[1, 2, 3], [4, 5, 6]]));
    }
}
