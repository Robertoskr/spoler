use crate::utils;
use crate::Task;
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
    // Entrypoint for the worker processes
    pub fn run(self, receiver: Receiver<Task>, app_settings: HashMap<String, String>) -> () {
        //is this application a python app ?
        let app =
            utils::get_string_from_settings(&app_settings, "--app".to_string(), "".to_string());

        match app.as_str() {
            "python" => {
                Self::_run_python(receiver, &app_settings);
            }
            _ => {
                tokio::spawn(async move {
                    self._run(receiver, &app_settings).await;
                });
            }
        }
    }

    /// Starts the worker process for python applications
    fn _run_python(mut receiver: Receiver<Task>, app_settings: &HashMap<String, String>) -> () {
        eprintln!("Starting execution of python application worker");
        pyo3::prepare_freethreaded_python();
        let python_guard = Python::acquire_gil();
        let python = python_guard.python();
        //get the main app
        let python_project_path = utils::get_string_from_settings(
            app_settings,
            "--project-path".to_string(),
            "".to_string(),
        );
        let code = utils::get_code_from_file(python_project_path.clone());

        let main_app =
            PyModule::from_code(python, code.as_str(), &python_project_path, "SpoolerApp")
                .unwrap()
                .getattr("SpoolerApp")
                .unwrap();

        //prepare all the things of the app
        //this will call the __init__ function of the python main app
        //if youare building the app, you should prepare your application there
        //like connection to db, etc...
        let main_app = main_app
            .call0()
            .expect("Error initializing the python application");

        loop {
            let message = receiver.try_recv();
            match message {
                Err(_) => continue,
                Ok(task) => {
                    eprintln!("Python Worker: got incoming task");

                    //since we have only one python thread, we are going to run each task in sync way,
                    //TODO: handle edge cases, and don't fail on error
                    let python_fn_name = task.settings.unwrap().executor_ref.unwrap();
                    main_app.call_method0(&python_fn_name).expect("");
                }
            }
        }
        //close the python application needed hooks
    }

    //run a normal app (requests, tcp tasks)
    async fn _run(self, mut receiver: Receiver<Task>, _: &HashMap<String, String>) -> () {
        loop {
            let message = receiver.recv().await;
            match message {
                Some(task) => {
                    println!("Worker: got incoming task");

                    //now process the task, the task should have enought information for knowing how it needs to be processed
                    //and the worker should follow that guidelines;
                    tokio::task::spawn(async move {
                        let _ = AsyncWorker::process_task(task).await;
                    });
                }
                None => (),
            }
        }
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
