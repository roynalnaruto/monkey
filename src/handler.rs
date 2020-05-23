use std::{thread, time};

use tokio::runtime::Handle;

#[derive(Debug)]
pub struct Handler {
    id: u8,
}

impl Handler {
    pub fn new(rt_handle: &Handle) -> Self {
        let handler = Handler { id: 3 };

        rt_handle.spawn(async {
            loop {
                let one_second = time::Duration::from_millis(1000);

                thread::sleep(one_second);
            }
        });

        handler
    }
}
