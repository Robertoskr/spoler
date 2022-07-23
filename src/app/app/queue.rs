use serde::Serialize;
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::{thread, time};

//some helper functions
fn parent_idx(idx: usize) -> usize {
    idx / 2
}

fn left_child(idx: usize) -> usize {
    idx * 2 + 1
}

fn right_child(idx: usize) -> usize {
    left_child(idx) + 1
}

pub struct BasicQueue<T> {
    queue: VecDeque<T>,
}

pub struct Heap<T> {
    comp: fn(&T, &T) -> bool,
    pub data: Vec<T>,
    size: usize,
}

pub trait Queue<T> {
    //create a new queue
    fn new() -> Self;
    //for seing if the queue is empty or not, and having a
    //an overview if the queue is full
    fn len(&self) -> usize;
    //for adding a new task to the queue
    fn insert(&mut self, task: T) -> ();
    //for seing what is the next task
    fn peek(&self) -> Option<&T>;
    //for getting and deleting the task from the queue
    fn pop(&mut self) -> Option<T>;
}

impl<T> Heap<T>
where
    T: std::cmp::PartialOrd + std::fmt::Debug,
{
    //creates a new heap with the default comparator
    //default comparator: less than
    pub fn new() -> Self {
        Self {
            comp: move |a: &T, b: &T| a < b,
            data: Vec::new(),
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    //adds a new entry to the heap
    pub fn insert(&mut self, new_entry: T) -> () {
        //for inserting, we add a new entry to the end of the queue and then, we find it's position
        self.data.push(new_entry);
        let mut entry_idx = self.size;
        self.size += 1;

        while entry_idx > 0 {
            let parent_idx = parent_idx(entry_idx);
            if (self.comp)(&self.data[entry_idx], &self.data[parent_idx]) {
                self.swap(entry_idx, parent_idx);
                entry_idx = parent_idx;
            } else {
                break;
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.size == 0 {
            return None;
        }

        Some(&self.data[0])
    }

    pub fn pop(&mut self) -> Option<T> {
        //swap the first with the last one
        if self.size == 0 {
            return None;
        }
        self.swap(0, self.size - 1);

        let result = self.data.pop().unwrap();
        self.size -= 1;

        //move down the heap the value that wee set as the first
        self.bubble_down(0);

        Some(result)
    }

    fn bubble_down(&mut self, idx: usize) -> () {
        let left_children_idx = left_child(idx);
        let right_children_idx = right_child(idx);
        if left_children_idx < self.size
            && (self.comp)(&self.data[left_children_idx], &self.data[idx])
        {
            self.swap(idx, left_children_idx);
            self.bubble_down(left_children_idx);
        }
        if right_children_idx < self.size
            && (self.comp)(&self.data[right_children_idx], &self.data[idx])
        {
            self.swap(idx, right_children_idx);
            self.bubble_down(right_children_idx);
        }
    }

    //swap the values of two nodes of the heap
    fn swap(&mut self, idx_a: usize, idx_b: usize) -> () {
        self.data.swap(idx_a, idx_b);
    }

    fn debug(&self) -> () {
        println!("Debugging heap");
        println!("DATA: {:?}", self.data);
        println!("SIZE: {:?}", self.size);
    }
}

impl<T> Queue<T> for BasicQueue<T> {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn insert(&mut self, task: T) -> () {
        self.queue.push_back(task);
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn peek(&self) -> Option<&T> {
        self.queue.front()
    }

    fn pop(&mut self) -> Option<T> {
        self.queue.pop_front()
    }
}
