mod app;
mod utils;
mod worker;

use app::{App, Heap, Task, TaskType};
use std::collections::HashMap;
use std::env;
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use worker::AsyncWorker;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening to port 8080");

    //get the application settings
    let app_settings: HashMap<String, String> = utils::get_app_settings(args);

    let (sender, receiver): (Sender<Task>, Receiver<Task>) = channel(10);

    //create the application
    let mut main_app: App<Heap<Task>> = App::new(sender);

    let n_queues: usize =
        utils::get_usize_from_settings(&app_settings, "--queues".to_string(), "1".to_string());

    for _ in 0..n_queues {
        main_app.add_new_empty_queue();
    }

    //create the worker, and start the worker in a different thread;
    tokio::spawn(async move {
        AsyncWorker {}.run(receiver, app_settings.clone()).await;
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
