struct QueueDescriptor {}

pub trait Worker {
    fn new() -> Self;
    fn start(&self) -> ();
}

pub struct SyncWorker {
    queue_descriptor: QueueDescriptor,
}

impl Worker for SyncWorker {
    fn new() -> Self {
        Self {
            queue_descriptor: QueueDescriptor {},
        }
    }

    //start a worker to start running the tasks of the queue descriptor
    fn start(&self) -> () {}
}
