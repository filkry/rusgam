use databucket::{SDataBucket};

pub struct SGameContext<'a> {
    pub cur_frame: u64,

    pub data_bucket: SDataBucket<'a>,
}
