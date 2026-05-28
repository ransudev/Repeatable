#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopMode {
    Once,
    Count(u32),
    Infinite,
}

pub fn next_loop(mode: &mut LoopMode) -> bool {
    match mode {
        LoopMode::Once => false,
        LoopMode::Infinite => true,
        LoopMode::Count(count) => {
            if *count > 1 {
                *count -= 1;
                true
            } else {
                *count = 0;
                false
            }
        }
    }
}
