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

    //get the application settings
    let app_settings: HashMap<String, String> = utils::get_app_settings(args);

    let port: usize =
        utils::get_usize_from_settings(&app_settings, "--port".to_string(), "8080".to_string());
    let host: String = utils::get_string_from_settings(
        &app_settings,
        "--host".to_string(),
        "localhost".to_string(),
    );

    //create the tcp listener
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to bind to tcp port");

    let (sender, receiver): (Sender<Task>, Receiver<Task>) = channel(100);

    //create the application
    let mut main_app: App<Heap<Task>> = App::new(sender);

    // how many queues we are going to have ?
    let n_queues: usize =
        utils::get_usize_from_settings(&app_settings, "--queues".to_string(), "1".to_string());

    for _ in 0..n_queues {
        main_app.add_new_empty_queue();
    }

    //start a new instance of the app (with same queues) for processing all the clients connections
    //TODO: create a new thread for this, and not just a new task
    tokio::spawn(async move {
        loop {
            let connection = listener.accept().await.unwrap();
            let mut app = main_app.clone();
            tokio::task::spawn(async move {
                app.run(connection).await;
            });
        }
    });

    // Run the worker async or sync depending on the application type
    // this blocks the thread until the execution is finished.
    AsyncWorker {}.run(receiver, app_settings.clone());
}
