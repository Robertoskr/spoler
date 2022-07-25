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
    repeat_interval: Option<u32>,
    //represents the times this task should be repeated
    retries: Option<i32>,
}

impl ops::Sub<i32> for TaskSettings {
    type Output = TaskSettings;

    fn sub(self, _rhs: i32) -> TaskSettings {
        let mut output = TaskSettings {
            repeat_interval: self.repeat_interval,
            retries: None,
        };
        if self.retries.is_some() {
            output.retries = Some(self.retries.unwrap() - 1);
        }
        output
    }
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub queue: usize,
    pub id: String,
    pub weight: Option<usize>,
    //example: 2022-07-25T13:30:9.15Z
    pub eta: Option<String>,
    pub payload: Option<String>,
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

    pub fn get_weight(&self) -> f64 {
        self.weight.unwrap_or(0) as f64
    }

    pub fn should_run_now(&self) -> bool {
        match &self.eta {
            Some(eta) => {
                let now = Utc::now();
                let eta = get_eta(Some(eta.clone()));
                if eta < now {
                    return now - eta < Duration::seconds(3);
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
            weight: self.weight.clone(),
            id: self.id.clone(),
            payload: self.payload.clone(),
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
