// All Used Windows

trait SDL2Window<T> {
    fn init() -> Result<T, String> {
        // Stuff

        let ret = Self::start()?;
        Ok(ret)
    }

    fn start() -> Result<T, String>;
}

pub mod GraphingWindow {
    pub struct Window {
        // todo
    }

    impl super::SDL2Window<Window> for Window {
        fn start() -> Result<Window, String> {
            Ok(Window {})
        }
    }

    pub fn init<'a>() -> Result<&'a Window, String> {
        Ok(&Window {})
    }
}
