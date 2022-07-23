use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::cmp::Ordering;

#[derive(Debug, Deserialize)]
pub struct Task {
    pub queue: usize,
    pub id: String,
    pub weight: usize,
    pub eta: usize,
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
        self.weight as f64
    }

    fn should_run_now(&self) -> bool {
        true
    }
}

impl _Task for &Task {
    fn from_str(raw_str: &str) -> Task {
        serde_json::from_str(raw_str).unwrap()
    }

    fn get_queue(&self) -> usize {
        self.queue
    }

    fn get_weight(&self) -> f64 {
        self.weight as f64
    }

    fn should_run_now(&self) -> bool {
        true
    }
}

impl std::cmp::PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl std::cmp::PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}
