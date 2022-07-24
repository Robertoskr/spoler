mod app;

use app::App;
use app::Heap;
use app::Task;
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening to port 8080");

    //create the application
    let mut main_app: App<Heap<Task>> = App::new();

    let n_queues: usize = args[1].parse().unwrap();

    for i in 0..n_queues {
        main_app.add_new_empty_queue();
    }
    //start the listener worker
    loop {
        let connection = listener.accept().await.unwrap();

        //when cloning we are just creating new references to the same queues
        let mut app = main_app.clone();
        tokio::spawn(async move {
            app.run(connection).await;
        });
    }
}
