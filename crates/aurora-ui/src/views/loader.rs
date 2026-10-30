//! Reads many projects at once without starting too many engine processes (spec 003
//! FR-006, FR-008, research R3): at most `limit` requests in flight, each answer applied
//! as it arrives, and an answer dropped when a newer request for the same project was
//! made or the project is no longer listed. A project is never read twice at once. Only the scheduling lives here; the view runs
//! the requests.

use std::collections::{HashMap, HashSet, VecDeque};

/// The most cycle state reads that run at the same time.
pub const MAX_PARALLEL_READS: usize = 4;

/// One read to run: the project and the token of this request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub id: String,
    token: u64,
}

#[derive(Debug)]
pub struct Loader {
    limit: usize,
    next_token: u64,
    /// The projects being read.
    running: HashSet<String>,
    queue: VecDeque<String>,
    /// The token of the latest request per project.
    latest: HashMap<String, u64>,
    listed: HashSet<String>,
}

impl Loader {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            next_token: 0,
            running: HashSet::new(),
            queue: VecDeque::new(),
            latest: HashMap::new(),
            listed: HashSet::new(),
        }
    }

    /// The projects shown now; queued reads of the others are dropped, and their answers
    /// will be.
    pub fn set_listed(&mut self, ids: impl IntoIterator<Item = String>) {
        self.listed = ids.into_iter().collect();
        let listed = &self.listed;
        self.queue.retain(|id| listed.contains(id));
        self.latest.retain(|id, _| listed.contains(id));
    }

    /// Asks for `ids` again; returns the reads to start now.
    pub fn request(&mut self, ids: impl IntoIterator<Item = String>) -> Vec<Request> {
        for id in ids {
            if !self.listed.contains(&id) {
                continue;
            }
            self.next_token += 1;
            self.latest.insert(id.clone(), self.next_token);
            if !self.queue.contains(&id) {
                self.queue.push_back(id);
            }
        }
        self.dispatch()
    }

    /// A read finished: whether its answer is still wanted, and the reads to start now.
    pub fn finish(&mut self, request: &Request) -> (bool, Vec<Request>) {
        self.running.remove(&request.id);
        let wanted = self.listed.contains(&request.id)
            && self.latest.get(&request.id) == Some(&request.token)
            && !self.queue.contains(&request.id);
        (wanted, self.dispatch())
    }

    fn dispatch(&mut self) -> Vec<Request> {
        let mut started = Vec::new();
        let mut waiting = VecDeque::new();
        while self.running.len() < self.limit {
            let Some(id) = self.queue.pop_front() else {
                break;
            };
            if self.running.contains(&id) {
                // Its earlier read still runs; this one starts when it ends.
                waiting.push_back(id);
                continue;
            }
            let Some(&token) = self.latest.get(&id) else {
                continue;
            };
            self.running.insert(id.clone());
            started.push(Request { id, token });
        }
        while let Some(id) = waiting.pop_back() {
            self.queue.push_front(id);
        }
        started
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("p{i}")).collect()
    }

    fn loader(n: usize) -> Loader {
        let mut loader = Loader::new(MAX_PARALLEL_READS);
        loader.set_listed(ids(n));
        loader
    }

    #[test]
    fn never_more_than_the_limit_in_flight_and_every_project_once() {
        let mut loader = loader(20);
        let mut running = loader.request(ids(20));
        let mut answered = Vec::new();
        let mut most = running.len();
        // Finish from the back, so answers arrive out of order.
        while let Some(done) = running.pop() {
            let (wanted, started) = loader.finish(&done);
            assert!(wanted);
            answered.push(done.id);
            running.extend(started);
            most = most.max(running.len());
        }
        assert_eq!(most, MAX_PARALLEL_READS);
        answered.sort();
        let mut expected = ids(20);
        expected.sort();
        assert_eq!(answered, expected);
    }

    #[test]
    fn an_older_answer_is_dropped() {
        let mut loader = loader(1);
        let first = loader.request(ids(1)).remove(0);
        // Refreshed while the first read runs: it waits for a free slot.
        assert!(loader.request(ids(1)).is_empty());
        let (wanted, started) = loader.finish(&first);
        assert!(!wanted, "the first answer is older than the refresh");
        assert_eq!(started.len(), 1);
        let (wanted, started) = loader.finish(&started[0]);
        assert!(wanted);
        assert!(started.is_empty());
    }

    #[test]
    fn answers_for_removed_projects_are_dropped() {
        let mut loader = loader(6);
        let running = loader.request(ids(6));
        assert_eq!(running.len(), 4);
        loader.set_listed(ids(6).into_iter().filter(|id| id != "p0" && id != "p5"));
        let (wanted, started) = loader.finish(&running[0]);
        assert!(!wanted);
        // p5 was queued and is not read any more; p4 still is.
        assert_eq!(
            started,
            vec![Request {
                id: "p4".into(),
                token: 5
            }]
        );
        assert!(loader.request(["p5".to_owned()]).is_empty());
    }

    #[test]
    fn a_zero_limit_still_reads() {
        let mut loader = Loader::new(0);
        loader.set_listed(ids(2));
        assert_eq!(loader.request(ids(2)).len(), 1);
    }
}
