use std::collections::{HashMap};

#[derive(Clone, Debug)]
pub struct RiderConfig {
    classes : HashMap<String, (u32, u32)>,   // Classes with sizes start bib, end bib inclusive
}


impl RiderConfig {
    pub fn new() -> Self {
        RiderConfig { 
            classes: HashMap::new() 
        }
    }

    pub fn set_class(&mut self, class_name : &str, start_bib : u32, end_bib : u32) -> Option<(u32, u32)> {
        self.classes.insert(class_name.to_string(), (start_bib, end_bib))
    }

    pub fn erase_class(&mut self, class_name : &str) -> Option<(String, (u32, u32))> {
        self.classes.remove_entry(class_name)
    }

    pub fn get_classes(&self) -> impl Iterator<Item = &String> {
        self.classes.keys()
    }

    pub fn get_class_dimensions(&self, class_name : &str) -> Option<&(u32, u32)> {
        self.classes.get(class_name)
    }
}