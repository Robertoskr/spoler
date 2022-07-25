mod app;
mod worker;

use app::{ApiTaskSettings, App, Heap, Task, TaskType};
use std::env;
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use worker::SyncWorker;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening to port 8080");

    let (sender, mut receiver): (Sender<Task>, Receiver<Task>) = channel(10);

    //create the application
    let mut main_app: App<Heap<Task>> = App::new(sender);

    let n_queues: usize = args[1].parse().unwrap();

    for i in 0..n_queues {
        main_app.add_new_empty_queue();
    }

    //create the worker, and start the worker in a different thread;
    let mut worker = SyncWorker::new(receiver);
    tokio::spawn(async move {
        worker.run().await;
    });
    //start a new instance of the app (with same queues) for processing all the clients connections
    loop {
        let connection = listener.accept().await.unwrap();
        let mut app = main_app.clone();
        tokio::spawn(async move {
            app.run(connection).await;
        });
    }
}
