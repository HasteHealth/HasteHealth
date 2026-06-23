#[allow(dead_code)]
pub fn csv() {}

#[cfg(test)]
mod tests {
    use csv;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename = "name", serialize_with = "serialize_vec_as_csv")]
        name: Vec<String>,
    }

    fn serialize_vec_as_csv<S>(vec: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let joined = vec.join(";");
        serializer.serialize_str(&joined)
    }

    #[test]
    fn test_many_csv() {
        let value = TestStruct {
            name: vec!["Alice".to_string(), "Bob".to_string()],
        };
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.serialize(&value).unwrap();
        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        assert_eq!(data, "name\nAlice;Bob\n");
    }
}
