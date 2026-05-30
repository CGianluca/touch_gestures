mod gestures;
mod configuration;

use input::{Libinput, LibinputInterface};
use libc::{O_ACCMODE, O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use serde::Deserialize;
use std::fs::read_to_string;
use std::sync::OnceLock;

struct Interface;
static CONFIGURATION: OnceLock<configuration::Configuration> = OnceLock::new();

use input::event::touch::*;
use input::Event::*;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_ACCMODE == O_RDONLY) | (flags & O_ACCMODE == O_RDWR))
            .write((flags & O_ACCMODE == O_WRONLY) | (flags & O_ACCMODE == O_RDWR))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        let _ = File::from(fd);
    }
}

fn main() {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();
    
    let mut gesture : Option<gestures::Gesture> = None;
    
    let json_settings = match read_to_string("/home/gianluca/Workspace/Coding/touch_gestures/config.json"){
        Ok(file ) => file,
        Err(error) => panic!("Error while reading settings file:\n {}", error),
    };

    let mut s: configuration::Configuration = match serde_json::from_str(&json_settings){
        Ok(s) => s,
        Err(error) => panic!("Error while parsing settings file:\n {}", error),
    };
    s.settings.vertical_angle = (s.settings.vertical_angle*std::f32::consts::PI/180.0).tan().abs();
    s.settings.horizontal_angle = (s.settings.horizontal_angle*std::f32::consts::PI/180.0).tan().abs();
    CONFIGURATION.set(s).expect("Error while setting up the configuration");

    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Touch(TouchEvent::Down(evt)) => {
                    match gesture {
                       Some(ref mut g) => {
                            match evt.slot() {
                                Some(s) => {
                                    if s>=1 {
                                        g.add_finger((evt.x() as u32, evt.y() as u32));
                                    }
                                },
                                None => {},
                            }
                       },
                       None => {
                            let mut g= gestures::Gesture::new();
                            g.add_finger((evt.x() as u32, evt.y() as u32));
                            gesture = Some(g);
                       }
                    }
                },
                Touch(TouchEvent::Up(evt)) => {
                    match gesture {
                        Some(ref mut g) => {
                            match evt.slot() {
                                Some(_s) => {
                                    g.remove_finger();
                                },
                                None => {},
                            }
                            if g.is_finished() {
                                g.resolve();
                                gesture = None;
                            }                                
                       },
                       None => {},
                    }

                },
                Touch(TouchEvent::Motion(evt)) => {
                    if let Some(ref mut g) = gesture {
                        g.add_move((evt.x() as u32, evt.y() as u32), evt.slot().expect("Expected slot") as usize);                        
                        if CONFIGURATION.get().unwrap().settings.hot_gesture {
                            if let Some(n) = g.get_movements_size() {
                                if n > CONFIGURATION.get().unwrap().settings.min_num_movements {
                                    g.resolve();
                                    gesture = None;
                                }
                            }
                        }
                    }
                },
                Touch(TouchEvent::Cancel(_evt)) => {
                },
                Touch(TouchEvent::Frame(_evt)) => {
                },
                _ => {}
            }
        }
    }
}