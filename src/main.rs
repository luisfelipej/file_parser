use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use serde_json::{Value, json};

fn transform_with_python(value: &Value) -> Result<Value, String> {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let code = r#"
def transform(value):
    return value + "!"
"#;
        let locals = PyDict::new(py);
        locals.set_item("value", PyString::new(py, &value.to_string())).unwrap();
        py.run(code, None, Some(locals)).map_err(|e| e.to_string())?;

        let transformed_value = locals.get_item("transform").unwrap()
                                      .call1((value.to_string(),)).map_err(|e| e.to_string())?
                                      .extract::<String>().map_err(|e| e.to_string())?;
        
        Ok(json!(transformed_value))
    })
}

fn main() -> PyResult<()> {
    let raw_data = r#"
        [
            {"name": "Alice", "age": 45},
            {"name": "Bob", "age": 25}
        ]
    "#;

    let mut data: Vec<Value> = serde_json::from_str(raw_data).expect("JSON was not well-formatted");
    let mut transformations : HashMap<&str, fn(&Value)-> Result<Value, String>> = HashMap::new();

    transformations.insert("age", transform_with_python);
    for user in &mut data {
        if let Value::Object(ref mut user) = user {
            for (key, transform) in &transformations {
                if let Some(value) = user.get(*key) {
                    match transform(value) {
                        Ok(new_value) => {user.insert(key.to_string(), new_value);},
                        Err(e) => {println!("Error transforming {}: {}", key, e)}
                    }
                }
            }
        }
    }
    println!("{:?}", data);
    Ok(())
}
