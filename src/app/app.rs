use std::clone::Clone;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

pub mod queue;
mod task;
mod worker;

pub use queue::Heap;
use queue::Queue;
pub use task::{Task, TaskSettings, TaskType};

type AppQueue<T> = Arc<Mutex<T>>;

pub struct App<T> {
    pub queues: Vec<AppQueue<T>>,
    pub sender: Sender<Task>,
}

impl<T> App<T>
where
    T: Queue<Task> + std::fmt::Debug,
{
    pub fn new(s: Sender<Task>) -> Self {
        Self {
            queues: Vec::new(),
            sender: s,
        }
    }

    //adds a new queue to the set of queues, and returns the position that it ocuppies in the list of queues
    //this can be used while declaring tasks, etc...

    pub fn add_new_empty_queue(&mut self) -> () {
        self.queues.push(Arc::new(Mutex::new(T::new())))
    }

    pub async fn run(&mut self, connection: (TcpStream, SocketAddr)) {
        println!("Acepted and running incoming connection: {}", connection.1);

        //create the reader that will be reading from the socket
        let (mut socket, _) = connection;
        let (read, _) = socket.split();
        let mut reader = BufReader::new(read);
        let mut buffer = String::new();

        loop {
            tokio::select! {
                tasks = self.poll_queues() => {
                    //send this tasks to the proper worker,
                    //each worker has a queue of tasks to execute in that moment
                    //what we can do now, is sending back the task to execute now via the tcp client,
                    //so the client knows that that task needs to be executed in that moment
                    //this is a TODO
                    for t in tasks {
                        let _ = self.sender.send(t).await;
                    }
                }
                bytes_read = reader.read_line(&mut buffer) => {
                    let bytes_read = bytes_read.unwrap();
                    if bytes_read == 0 {
                        return;
                    }
                    let raw_task = std::str::from_utf8(&buffer.as_bytes())
                        .expect("Invalid data type")
                        .trim();

                    //create the task from the raw input
                    //send the task to the appropiate queue
                    let task: Task = Task::from_str(raw_task);

                    //get the lock of the queue, and insert the new task
                    let queue_idx = task.get_queue();
                    self.queues[queue_idx].lock().await.insert(task);

                    //debug
                    //println!("{:?}", self.queues[queue_idx].lock().unwrap());

                    //clean the buffer for the next message
                    buffer.clear();
                }
            };
        }
    }

    pub async fn poll_queues(&mut self) -> Vec<Task> {
        //sleep for some time, for now burning the thread
        tokio::time::sleep(Duration::from_nanos(200)).await;
        let mut result: Vec<Task> = Vec::new();
        for i in 0..self.queues.len() {
            let mut should_run = false;
            let mut should_reschedule = false;
            //do this inside a block so the lock is released, and other can use it
            {
                let queue_lock = &self.queues[i].lock().await;
                let task = queue_lock.peek();
                if task.is_some() && task.unwrap().should_run_now() {
                    should_run = true;
                    should_reschedule = task.unwrap().should_reschedule();
                }
            }
            if should_run {
                //this is ok, because should pop is true
                let mut queue_lock = self.queues[i].lock().await;
                let task = queue_lock.pop().unwrap();

                if should_reschedule {
                    queue_lock.insert(task.get_next());
                }

                //add this task to the result, should be run now
                result.push(task);
            }
        }
        result
    }
}

impl<T> Clone for App<T> {
    fn clone(&self) -> Self {
        let mut queues: Vec<AppQueue<T>> = Vec::new();
        for q in &self.queues {
            queues.push(Arc::clone(q));
        }
        Self {
            queues: queues,
            sender: self.sender.clone(),
        }
    }
}
