
pub fn get_word_position_in_data(mut index: usize) -> (usize, usize) {
    let k = index / 3;
    index %= 3;
    let mut offset = k * (20 + 32 + 32);
    let size;
    match index {
        0 => {
            size = 20;
        }
        1 => {
            offset += 20;
            size = 32;
        }
        2 => {
            offset += 20 + 32;
            size = 32
        }
        _ => unreachable!()
    };
    (offset, size)
}
