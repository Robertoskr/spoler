use crate::ApiTaskSettings;
use crate::Task;
use crate::TaskType;
use tokio::sync::mpsc::Receiver;

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

                    //now process the task, the task should have enought information for knowing how it needs to be processed
                    //and the worker should follow that guidelines;
                    self.process_task(task);
                }
                None => (),
            }
        }
    }

    //to do, do this asynchronously ?
    fn process_task(&self, task: Task) -> () {
        match task.task_type {
            // this means that the task needs to be send to some api
            // the api is located in the specific settings
            // and send to that api the provided payload
            TaskType::Api => {
                let ApiTaskSettings {
                    headers,
                    method,
                    content_type,
                    url,
                } = serde_json::from_str(task.specific_settings.as_str()).unwrap();
                // we have all the data, now, we need to make the request
                // use reqwest as a library for that .
            }
            // this means that the task needs to be send to some tcp address
            // all the info is located in the specific settings
            TaskType::Tcp => {
                //TODO implement this.
            }
        }
    }
}
