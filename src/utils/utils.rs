use std::collections::VecDeque;

pub struct MovingAverage {
    queue: VecDeque<i32>,
    max_size: usize,
}

impl MovingAverage {
    pub fn new(max_size: usize) -> Self {
        let queue: VecDeque<i32> = VecDeque::new();

        MovingAverage { queue, max_size }
    }

    pub fn add(&mut self, item: i32) {
        if self.queue.len() > self.max_size - 1 {
            self.queue.pop_front();
        }

        self.queue.push_back(item);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn sum(&self) -> f64 {
        self.queue.iter().sum::<i32>() as f64
    }

    pub fn mean(&self) -> f64 {
        if self.queue.len() > 0 {
            self.queue.iter().sum::<i32>() as f64 / self.queue.len() as f64
        } else {
            0.0
        }
    }
}
