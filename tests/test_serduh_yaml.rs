// Round-trip tests for serduh-yaml integration

use serduh_core::{Serialize, Serializer, SerializeStruct};
use serduh_yaml::YamlSerializer;
use yaml_parse::{parse, Value as YamlValue};

#[derive(Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

impl Serialize for Person {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Person", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("age", &self.age)?;
        state.end()
    }
}

fn main() {
    // Test 1: Simple scalar serialization
    println!("Test 1: Scalar serialization");
    let yaml = YamlSerializer::serialize(&"hello".to_string()).unwrap();
    assert_eq!(yaml, YamlValue::String("hello".to_string()));
    println!("  ✓ String serializes correctly");
    
    // Test 2: Number serialization
    println!("Test 2: Number serialization");
    let yaml = YamlSerializer::serialize(&42u32).unwrap();
    assert_eq!(yaml, YamlValue::Number(42));
    println!("  ✓ Number serializes correctly");
    
    // Test 3: Boolean serialization
    println!("Test 3: Boolean serialization");
    let yaml = YamlSerializer::serialize(&true).unwrap();
    assert_eq!(yaml, YamlValue::Bool(true));
    println!("  ✓ Boolean serializes correctly");
    
    // Test 4: List serialization
    println!("Test 4: List serialization");
    let items = vec!["a".to_string(), "b".to_string()];
    let yaml = YamlSerializer::serialize(&items).unwrap();
    match yaml {
        YamlValue::List(_) => println!("  ✓ List serializes correctly"),
        _ => panic!("Expected list"),
    }
    
    // Test 5: Struct serialization
    println!("Test 5: Struct serialization");
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    let yaml = YamlSerializer::serialize(&person).unwrap();
    match yaml {
        YamlValue::Map(pairs) => {
            assert_eq!(pairs.len(), 2);
            println!("  ✓ Struct serializes correctly");
        }
        _ => panic!("Expected map"),
    }
    
    // Test 6: YAML parsing
    println!("Test 6: YAML parsing");
    let yaml_str = "name: Alice\nage: 30";
    let parsed = parse(yaml_str).unwrap();
    match parsed {
        YamlValue::Map(pairs) => {
            assert_eq!(pairs.len(), 2);
            println!("  ✓ YAML parses correctly");
        }
        _ => panic!("Expected map"),
    }
    
    println!("\n✅ All serduh-yaml integration tests passed!");
}
