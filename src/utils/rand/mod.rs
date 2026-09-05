
pub fn uuid() -> String {
	uuid::Uuid::new_v4().to_string()
}

pub fn nano(len: usize) -> String {
	nanoid::nanoid!(len)
}