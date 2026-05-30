use std::collections::HashMap;

#[derive(super::Deserialize, Clone, Debug)]
pub struct Configuration {
    pub settings: Settings,
    commands: GestureTypeConfig
}

impl Configuration {
    pub fn get_command(&self, gesture : &crate::gestures::GestureTypes) -> Option<&String> {
        match gesture {
            crate::gestures::GestureTypes::TwoFingers(shape) => {
                (shape != &crate::gestures::GestureShape::None).then(|| &self.commands.two_fingers.commands[shape]).filter(|s| !s.is_empty())
            },
            crate::gestures::GestureTypes::ThreeFingers(shape) => {
                (shape != &crate::gestures::GestureShape::None).then(|| &self.commands.three_fingers.commands[shape]).filter(|s| !s.is_empty())
            }
            crate::gestures::GestureTypes::FourFingers(shape) => {
                (shape != &crate::gestures::GestureShape::None).then(|| &self.commands.four_fingers.commands[shape]).filter(|s| !s.is_empty())
            }
            crate::gestures::GestureTypes::FiveFingers(shape) => {
                (shape != &crate::gestures::GestureShape::None).then(|| &self.commands.five_fingers.commands[shape]).filter(|s| !s.is_empty())
            }
            _ => None
        }    
    }
}

#[derive(crate::Deserialize, Clone, Debug)]
struct GestureTypeConfig {
   two_fingers: ShapeConfig,
   three_fingers: ShapeConfig,
   four_fingers: ShapeConfig,
   five_fingers: ShapeConfig
}

#[derive(crate::Deserialize, Clone, Debug)]
pub struct Settings {
    pub vertical_angle: f32,
    pub horizontal_angle: f32,
    pub error_treshold: f32,
    pub min_num_movements: usize,
    pub hot_gesture: bool,
}

#[derive(crate::Deserialize, Clone, Debug)]
struct ShapeConfig {
    commands: HashMap<crate::gestures::GestureShape, String>,
}
