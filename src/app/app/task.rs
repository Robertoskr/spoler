use chrono::prelude::*;
use chrono::Duration;
use serde::Deserialize;
use serde_json;
use std::cmp::Ordering;
use std::ops;
use std::ops::Add;

#[derive(Debug, Deserialize, Clone)]
pub struct TaskSettings {
    //represents the seconds of the interval in wich this task should be repeated
    pub repeat_interval: Option<u32>,
    //represents the times this task should be repeated
    pub retries: Option<i32>,
    pub url: Option<String>,
    pub headers: Option<String>,
    pub method: Option<String>,
    pub function_path: Option<String>,
}

//this determines how the task is going to be resolved
#[derive(Debug, Deserialize, Clone)]
pub enum TaskType {
    //the task is resolved via api
    Api = 1,
    //the task is resolved sending a tcp message
    Tcp = 2,
    //the task is resolved via a python function 
    Python = 3,
    //we don't know more yet... (WIP)
    Other = 4,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    //in wich queue this is going to be in
    pub queue: usize,
    pub id: String,
    //example: 2022-07-30T09:44:9.15Z
    pub eta: Option<String>,
    pub task_type: i32,
    //the payload that we are going when processing this task
    pub payload: Option<String>,
    //the specific settings is a string in json format,
    //and need to have one format or other format depending of the type of task
    pub settings: Option<TaskSettings>,
}

impl Task {
    //parse a raw task to a task structure
    pub fn from_str(raw_str: &str) -> Self {
        serde_json::from_str(raw_str).unwrap()
    }

    pub fn get_queue(&self) -> usize {
        self.queue
    }

    pub fn should_run_now(&self) -> bool {
        match &self.eta {
            Some(eta) => {
                let now = Utc::now();
                let eta = get_eta(Some(eta.clone()));
                if eta < now {
                    return true;
                }
                return eta - now < Duration::seconds(3);
            }
            None => (),
        };
        true
    }

    pub fn should_reschedule(&self) -> bool {
        //no settings we do nothing
        if self.settings.is_none() {
            return false;
        }

        let settings = self.settings.as_ref().unwrap();
        let repeat_interval = settings.repeat_interval.unwrap_or(0);
        //no repeat interval, we do nothing
        if repeat_interval == 0 {
            return false;
        }

        //if retries is 0 we finished our work
        let retries = settings.retries.unwrap_or(-1);
        if retries == -1 || retries > 0 {
            return true;
        }
        false
    }

    pub fn get_next(&self) -> Task {
        Task {
            eta: self.get_next_eta(),
            queue: self.queue.clone(),
            id: self.id.clone(),
            payload: self.payload.clone(),
            task_type: self.task_type.clone(),
            settings: Some(self.settings.clone().unwrap() - 1),
        }
    }

    fn get_next_eta(&self) -> Option<String> {
        if self.eta.is_none() {
            return None;
        }
        let eta = get_eta(self.eta.clone());
        //todo refactor this
        Some(
            eta.add(Duration::seconds(
                self.settings
                    .as_ref()
                    .unwrap()
                    .repeat_interval
                    .unwrap()
                    .into(),
            ))
            .to_string(),
        )
    }
}

impl std::cmp::PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        let left_eta = get_eta(self.eta.clone());
        let right_eta = get_eta(other.eta.clone());
        left_eta < right_eta
    }
}

impl std::cmp::PartialOrd for Task {
    //compare by eta
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left_eta = get_eta(self.eta.clone());
        let right_eta = get_eta(other.eta.clone());
        Some(left_eta.cmp(&right_eta))
    }
}

fn get_eta(eta: Option<String>) -> DateTime<Utc> {
    return if eta.is_some() {
        eta.unwrap().parse::<DateTime<Utc>>().unwrap()
    } else {
        Utc::now()
    };
}

impl ops::Sub<i32> for TaskSettings {
    type Output = TaskSettings;

    fn sub(self, _rhs: i32) -> TaskSettings {
        let mut output = TaskSettings {
            repeat_interval: self.repeat_interval,
            retries: None,
            url: self.url,
            method: self.method,
            headers: self.headers,
            function_path: self.function_path,
        };
        if self.retries.is_some() {
            output.retries = Some(self.retries.unwrap() - 1);
        }
        output
    }
}
