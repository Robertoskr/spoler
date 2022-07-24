use chrono::prelude::*;
use chrono::Duration;
use serde::Deserialize;
use serde_json;
use std::cmp::Ordering;

#[derive(Debug, Deserialize)]
pub struct Task {
    pub queue: usize,
    pub id: String,
    pub weight: Option<usize>,
    pub eta: Option<String>,
    pub payload: Option<String>,
}

pub trait _Task {
    fn from_str(string: &str) -> Task;
    fn get_queue(&self) -> usize;
    fn get_weight(&self) -> f64;
    fn should_run_now(&self) -> bool;
}

impl _Task for Task {
    //parse a raw task to a task structure
    fn from_str(raw_str: &str) -> Self {
        serde_json::from_str(raw_str).unwrap()
    }

    fn get_queue(&self) -> usize {
        self.queue
    }

    fn get_weight(&self) -> f64 {
        self.weight.unwrap_or(0) as f64
    }

    fn should_run_now(&self) -> bool {
        match &self.eta {
            Some(eta) => {
                let now = Utc::now();
                let eta = Utc
                    .datetime_from_str(eta.as_str(), "%Y-%m-%d %H:%M:%S")
                    .unwrap();
                if eta < now {
                    return now - eta < Duration::seconds(3);
                }
                return eta - now < Duration::seconds(3);
            }
            None => (),
        };
        true
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
        Utc.datetime_from_str(eta.unwrap().as_str(), "%Y-%m-%d %H:%M:%S")
            .unwrap()
    } else {
        Utc::now()
    };
}
