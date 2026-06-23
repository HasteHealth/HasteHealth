#[cfg(test)]
mod tests {
    use csv;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        name: Vec<String>,
    }

    #[test]
    fn test_many_csv() {
        /// Test for multi value csv
        let value = TestStruct {
            name: vec!["Alice".to_string(), "Bob".to_string()],
        };
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.serialize(&value).unwrap();
        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        assert_eq!(data, "name\nAlice,Bob\n");
    }
}
