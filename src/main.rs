mod app;

use app::App;
use app::BasicQueue;
use app::Task;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening to port 8080");

    //create the application
    let mut main_app: App<BasicQueue<Task>> = App::new();

    //create 10 queues
    //TODO: define the number of queues in a config file
    let n_queues = 10;
    for i in 0..n_queues {
        main_app.add_new_empty_queue();
    }
    //start the listener worker
    loop {
        let connection = listener.accept().await.unwrap();
        let mut app = main_app.clone();
        tokio::spawn(async move {
            app.run(connection).await;
        });
    }
}
