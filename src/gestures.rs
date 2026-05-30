use std::process::Command;
use std::env;

pub struct Gesture{
    pending_fingers : i32,
    number_finger : u32,
    gesture_type : GestureTypes,
    movements : Vec<Vec<(u32, u32)>>,
} 

pub enum GestureTypes{
   None,
   SingleFingers(GestureShape),
   TwoFingers(GestureShape),
   ThreeFingers(GestureShape),
   FourFingers(GestureShape),
   FiveFingers(GestureShape)
}

impl GestureTypes{
    pub fn set_shape(&mut self, shape: GestureShape) -> &Self {
        
        match self {
            GestureTypes::SingleFingers(s) => {*s = shape;},
            GestureTypes::TwoFingers(s) => {*s = shape;},
            GestureTypes::ThreeFingers(s) => {*s = shape;},
            GestureTypes::FourFingers(s) => {*s = shape;},
            GestureTypes::FiveFingers(s) => {*s = shape;},
            _ => {}
        }

        self
    }
}

#[derive(crate::Deserialize, Hash, Eq, PartialEq, Debug, Clone)]
pub enum GestureShape{
    None,
    Left,
    Top,
    Right,
    Bottom
}

impl Gesture {
    pub fn new() -> Self {
        Gesture{
            pending_fingers: 0,
            number_finger: 0,
            gesture_type: GestureTypes::None,
            movements: vec![],
        }
    }

    pub fn add_finger(&mut self, position : (u32, u32)) -> &Self {
        self.gesture_type = match self.gesture_type {
            GestureTypes::None => GestureTypes::SingleFingers(GestureShape::None),
            GestureTypes::SingleFingers(_) => GestureTypes::TwoFingers(GestureShape::None),
            GestureTypes::TwoFingers(_) => GestureTypes::ThreeFingers(GestureShape::None),
            GestureTypes::ThreeFingers(_) => GestureTypes::FourFingers(GestureShape::None),
            GestureTypes::FourFingers(_) => GestureTypes::FiveFingers(GestureShape::None),
            GestureTypes::FiveFingers(_) => GestureTypes::FiveFingers(GestureShape::None),
        };
        self.pending_fingers += 1;
        self.number_finger += 1;
        
        let m_f : Vec<(u32, u32)> = vec![position];
        self.movements.push(m_f);
        self
    }

    pub fn add_move(&mut self, position : (u32, u32), i : usize) -> &Self {
        self.movements[i].push(position);
        self
    }
    
    pub fn compute(&mut self) -> Option<&mut Self> {

        self//.movements
        // .iter()
        // .max_by_key(|p| p.len()).map(|v| v.len())
        .get_movements_size()
        .filter(|n| *n>CONFIGURATION.get().unwrap().settings.min_num_movements)?;

        let shape : Option<GestureShape> = self.movements
        .iter()
        .map(|v| Self::is_line(v))
        .collect::<Option<Vec<f32>>>().and_then(|v| {
            let mean_m : f32 = v.iter().sum::<f32>()/(v.len() as f32);
            // Compute SHAPE
            match mean_m {
                m if m < CONFIGURATION.get()?.settings.horizontal_angle => Some(Self::get_horizontal_direction(&self.movements[0])),
                m if m > CONFIGURATION.get()?.settings.vertical_angle => Some(Self::get_vertical_direction(&self.movements[0])),
                _ => None
            }
        });
        
        shape.map(|shape| {
            self.gesture_type.set_shape(shape);
            self 
        })
        
    }

    pub fn resolve(&mut self) {
        let cmd = self.compute()
        .and_then(|gesture| CONFIGURATION.get()?.get_command(gesture.gesture_type()));
        
        // println!("{}", self);
        if let Some(cmd) = cmd {
            let prg = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .env("PATH", env::var("PATH")
            .unwrap_or_default())
            .output();

            match prg {
                Ok(_) => {},
                Err(err) => println!("Error executing \"{}\": {}", cmd, err)
            }
        }
    }
    
    fn get_horizontal_direction(v: &Vec<(u32, u32)>) -> GestureShape {
        if v[0].0 > v[v.len() - 1].0 {
            GestureShape::Left
        } else {
            GestureShape::Right
        }
    }

    fn get_vertical_direction(v: &Vec<(u32, u32)>) -> GestureShape {
        if v[0].1 > v[v.len() - 1].1 {
            GestureShape::Top
        } else {
            GestureShape::Bottom
        }
    }       

    pub fn get_movements_size(& self) -> Option<usize> {
        self.movements
        .iter()
        .max_by_key(|p| p.len()).map(|v| v.len())
    }

    fn is_line(f: &Vec<(u32, u32)>) -> Option<f32> {
        let a = f[0];
        let b = f[f.len()-1];
        
        // m = (y1 - y2)/(x1 - x2)
        let m : f32;
        let c: f32;
        if a.0 == b.0 {
            m = 0.0;
            c = a.0 as f32;
        } else {
            m = (a.1 as f32 - b.1 as f32)/(a.0 as f32 - b.0 as f32);
            c = b.1 as f32 - m*(b.0 as f32);
        }

        let err: f32 = (f.iter().map(|x| {(x.1 as f32 - (x.0 as f32)*m - c).powf(2.0)}).sum::<f32>() / (f.len() as f32)).sqrt();
        
       /*  println!("Linea approssimata -> m={}   c={}", m, c);
        println!("Errore: {}", err);
        println!(""); */

        if err > CONFIGURATION.get()?.settings.error_treshold {
            None
        } else {
            Some(m. abs())
        }
    
    }

    pub fn is_finished(&mut self) -> bool {
        self.pending_fingers == 0
    }
    
    pub fn remove_finger(&mut self) {
        self.pending_fingers -= 1;
    }
    
    pub fn gesture_type(&self) -> &GestureTypes {
        &self.gesture_type
    }
    
}


use std::fmt::Write;

use crate::CONFIGURATION;

impl std::fmt::Display for Gesture {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg: String = String::from("Gesture is of type: ");
        /* match self.gesture_type{
            GestureTypes::SingleFingers(_) => msg += "SingleFinger",
            GestureTypes::TwoFingers(_)    => msg += "TwoFinger",
            GestureTypes::ThreeFingers(_)  => msg += "ThreeFinger",
            GestureTypes::FourFingers(_)   => msg += "FourFinger",
            GestureTypes::FiveFingers(_)   => msg += "FiveFinger",
            _ => {}
        }
        msg += "\n"; */
        
        write!(msg, "{}", self.gesture_type).unwrap();

/*             msg += "Shape of the gesture: ";
        match self.gesture_type(s) {
            GestureShape::Left => msg+="LEFT",
            GestureShape::Top => msg+="TOP",
            GestureShape::Right => msg+="RIGHT",
            GestureShape::Bottom => msg+="BOTTOM",
            _ => msg+="Undefined",
        }
        msg += "\n"; */
        
        let n_pos = self.movements.iter().max_by_key(|p| p.len()).expect("Error").len();
        write!(msg, " number of movements: {}", n_pos).unwrap();

        let enable_position_write : bool = false;
        if enable_position_write{
            msg += "\n";
            let n_pos = self.movements.iter().max_by_key(|p| p.len()).expect("Error").len();
            for i in 0..n_pos {
                for p_v in &self.movements{
                    match p_v.get(i){
                        Some(&v) => write!(msg, "({:^3}, {:^3})  ", v.0, v.1).unwrap(),
                        None => msg+="            ",
                    };
                }                
                msg += "\n"
            } 
        }
        write!(f,"{}", msg)
    }
}

impl std::fmt::Display for GestureShape {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let text = match self {
            GestureShape::None => "None",
            GestureShape::Left => "Left",
            GestureShape::Top => "Top",
            GestureShape::Right => "Right",
            GestureShape::Bottom => "Bottom"
        };

        write!(f, "{text}")
    }
}

impl std::fmt::Display for GestureTypes {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureTypes::None => write!(f,"None"),
            GestureTypes::SingleFingers(shape) => write!(f,"Type [SingleFinger] Shape [{}]", shape),
            GestureTypes::TwoFingers(shape) => write!(f,"Type [TwoFinger] Shape [{}]", shape),
            GestureTypes::ThreeFingers(shape) => write!(f,"Type [ThreeFinger] Shape [{}]", shape),
            GestureTypes::FourFingers(shape) => write!(f,"Type [FourFinger] Shape [{}]", shape),
            GestureTypes::FiveFingers(shape) => write!(f,"Type [FiveFinger] Shape [{}]", shape),
        }
    }

}
