

pub trait MaxThreshold {
    fn get(&mut self) -> u8;
}

pub struct LocalMaxThreshold(u8);

impl MaxThreshold for LocalMaxThreshold {
    fn get(&mut self) -> u8 {
        self.0
    }
}
