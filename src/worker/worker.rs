use crate::utils;
use crate::Task;
use crate::TaskType;
use pyo3::prelude::*;
use reqwest::header::HeaderMap;
use reqwest::Method;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub trait Worker {
    fn new() -> Self;
    fn start(&self) -> ();
}

pub struct AsyncWorker {}

impl AsyncWorker {
    //start a worker to start running the tasks of the queue descriptor
    pub async fn run(
        self,
        mut receiver: Receiver<Task>,
        app_settings: HashMap<String, String>,
    ) -> () {
        let is_python_app =
            utils::get_string_from_settings(&app_settings, "--app".to_string(), "".to_string())
                == "python".to_string();

        //if we are using a python app, we should start the python project app class
        if is_python_app {
            self._run_python(receiver, &app_settings).await;
        }
        self._run(receiver, &app_settings).await;
    }

    //run the worker used for python apps
    async fn _run_python(receiver: Receiver<Task>, &app_settings: HashMap<String, String>) -> () {
        Python::with_gil(|python| {
            loop {
                let message = receiver.recv().await;
                match message {
                    Some(task) => {
                        println!("Worker: got incoming task");

                        //now process the task, the task should have enought information for knowing how it needs to be processed
                        //and the worker should follow that guidelines;
                        tokio::task::spawn(async move {
                            AsyncWorker::process_task(task).await;
                        });
                    }
                    None => (),
                }
            }
        });
    }

    //run a normal app (requests, tcp tasks)
    async fn _run(self, mut receiver: Receiver<Task>, app_settings: HashMap<String, String>) -> () {
        loop {
            let message = receiver.recv().await;
            match message {
                Some(task) => {
                    println!("Worker: got incoming task");

                    //now process the task, the task should have enought information for knowing how it needs to be processed
                    //and the worker should follow that guidelines;
                    tokio::task::spawn(async move {
                        AsyncWorker::process_task(task).await;
                    });
                }
                None => (),
            }
        }
    }

    fn get_main_python_app(app_settings: &HashMap<String, String>, python: &Python) -> &PyAny {
        let python_project_path = utils::get_string_from_settings(
            app_settings,
            "--project-path".to_string(),
            "".to_string(),
        );

        if python_project_path.len() == 0 {
            return;
        }

        python
            .import(python_project_path.as_str())
            .unwrap()
            .getattr("SpoolerApp")
            .unwrap()
    }

    //to do, do this asynchronously ?
    pub async fn process_task(task: Task) -> Result<(), String> {
        match task.task_type {
            /*TaskType::Api*/
            1 => Self::process_request_task(task).await,
            /*TaskType::Tcp */
            2 => Ok(()),
            /*TaskType::Python*/
            3 => Err(String::from("Error, python tasks should run inside a python app, use the --app python flag to run that")),
            /*TaskType::Other */
            _ => Ok(()),
        }
    }

    async fn process_python_task(task: Task) -> Result<(), String> {
        Ok(())
    }

    //Process a request task
    //A request Task is a task that needs to be resolved calling an external api
    async fn process_request_task(task: Task) -> Result<(), String> {
        // we have all the data, now, we need to make the request
        // use reqwest as a library for that .
        if task.settings.is_none() {
            return Err(String::from("TaskType api must contain settings"));
        }
        let task_settings = task.settings.as_ref().unwrap();

        //get the headers
        let cloned_headers = task_settings.headers.clone().unwrap_or(String::from("{}"));
        let headers: HashMap<&str, &str> = serde_json::from_str(cloned_headers.as_str()).unwrap();
        let headers = get_headers(headers);
        let method = task_settings.method.clone().unwrap_or(String::new());
        let url = task_settings.url.clone().unwrap_or(String::new());

        //TODO: use a more low-level library like hyper for example for this
        match reqwest::Client::new()
            .request(get_method(method), url)
            .headers(headers)
            .json(&task.payload.unwrap())
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to send request: {}", e).to_string()),
        }
    }
}

fn get_headers(headers: HashMap<&str, &str>) -> HeaderMap {
    headers
        .into_iter()
        .map(|(name, value)| (name.parse().unwrap(), value.parse().unwrap()))
        .collect::<HeaderMap>()
}

fn get_method(method: String) -> Method {
    return match method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "DELETE" => Method::DELETE,
        "PUT" => Method::PUT,
        "PATCH" => Method::PATCH,
        _ => Method::OPTIONS,
    };
}
