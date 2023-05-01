use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use log::debug;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::from(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        debug!("Starting thread pool with {} workers", size);

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => job(),
                Err(_) => {
                    debug!("Worker {id} shutting down");
                    break;
                }
            };
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::ThreadPool;

    #[test]
    fn it_coordinates_worker_threads() {
        let num_threads = 4;
        let pool = ThreadPool::new(num_threads);

        let counter = Arc::new(Mutex::new(0));

        for _ in 0..num_threads {
            let counter = Arc::clone(&counter);
            pool.execute(move || (*counter.lock().unwrap()) += 1);
        }

        // By dropping thread pool, worker threads are joined.
        drop(pool);

        assert_eq!(num_threads, *counter.lock().unwrap());
    }
}
