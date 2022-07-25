use crate::Task;
use tokio::sync::mpsc::Receiver;

struct QueueDescriptor {}

pub trait Worker {
    fn new() -> Self;
    fn start(&self) -> ();
}

pub struct SyncWorker {
    receiver: Receiver<Task>,
}

impl SyncWorker {
    pub fn new(rcv: Receiver<Task>) -> Self {
        Self { receiver: rcv }
    }

    //start a worker to start running the tasks of the queue descriptor
    pub async fn run(&mut self) -> () {
        loop {
            let message = self.receiver.recv().await;
            match message {
                Some(task) => {
                    println!("Worker: got task: {:?}", task);

                    //now process the task, the task should know how to process it self
                    //and the worker should follow that guidelines;
                }
                None => (),
            }
        }
    }
}
