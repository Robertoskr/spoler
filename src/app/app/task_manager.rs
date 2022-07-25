use crate::Task;

pub struct TaskManager {}

impl TaskManager {
    pub fn new() -> Self {
        Self {}
    }
    pub fn process(&self, t: Task) -> () {
        //TODO
        //process the task now,
        //process is collecting some stats, and sending the task to the correct task Pool
        //do all the process in an async way since this is blocking other jobs
        println!("{:?}", t);
        ()
    }
}
