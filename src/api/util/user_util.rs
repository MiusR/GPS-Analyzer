use fake::{Fake, faker::name::raw::Name, locales::EN};


#[allow(dead_code)]
struct UserUtil;

#[allow(dead_code)]
impl UserUtil {
    pub fn get_randomly_generated_name() -> String {
        let random_name : String = Name(EN).fake();
        random_name.replace(" ", "_")
    }
}