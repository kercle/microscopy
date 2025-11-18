use std::collections::HashMap;

use interface::ws::compute_node::{Input, Output, Procedure};

struct InputData {
    image_stack: String,
}

pub struct FocusStacking {
    input_data: InputData,
}

impl FocusStacking {
    pub fn _new() -> Self {
        FocusStacking {
            input_data: InputData {
                image_stack: String::new(),
            },
        }
    }

    pub fn describe() -> Procedure {
        Procedure {
            display_name: "Focus Stacking".to_string(),
            description: "Generates an image with extended depth of field by combining multiple images taken at different focus distances.".to_string(),
            inputs: HashMap::from([
                ("image_stack".to_string(), Input::Selection {
                    display_name: "Image stack".to_string(),
                    options: vec![
                        "a".to_string(),
                        "b".to_string(),
                        "c".to_string()
                    ],
                    value: "a".to_string(),
                }),
            ]),
            outputs: HashMap::from([
                ("image".to_string(), Output::Image {
                    display_name: "Stacked Image".to_string(),
                }),
            ]),
        }
    }
}
