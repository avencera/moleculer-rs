use std::cmp::Eq;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug)]
pub struct QueueSet<T> {
    set: HashSet<T>,
    queue: VecDeque<T>,
}

impl<T> QueueSet<T>
where
    T: Eq + Hash + Clone,
{
    pub(crate) fn new(set: HashSet<T>, queue: VecDeque<T>) -> Self {
        Self { set, queue }
    }

    pub(crate) fn insert(&mut self, item: T) {
        if self.set.get(&item).is_none() {
            self.set.insert(item.clone());
            self.queue.push_back(item);
        }
    }

    pub(crate) fn remove(&mut self, item: &T) {
        if self.set.remove(&item) {
            self.queue
                .iter()
                .position(|x| x == item)
                .map(|index| self.queue.remove(index));
        }
    }

    pub(crate) fn len(&mut self) -> usize {
        self.queue.len()
    }

    pub(crate) fn iter(&self) -> std::collections::vec_deque::Iter<T> {
        self.queue.iter()
    }

    pub(crate) fn get_round_robin(&mut self) -> Option<T> {
        let item = self.queue.pop_front()?;
        self.queue.push_back(item.clone());
        Some(item)
    }
}

mod macros {
    #[macro_export]
    macro_rules! qset{
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(qset!(@single $rest)),*]));

    ($($value:expr,)+) => { qset!($($value),+) };
    ($($value:expr),*) => {
        {
            let _cap = qset!(@count $($value),*);
            let mut _set = ::std::collections::HashSet::with_capacity(_cap);
            let mut _queue = ::std::collections::VecDeque::with_capacity(_cap);

            $(
                _queue.push_back($value.clone());
                _set.insert($value);
            )*

            $crate::data_structures::QueueSet::new(_set, _queue)
        }
    };
    ($value:expr;$count:expr) => {
        {
            let c = $count;
            let mut _queue = ::std::collections::VecDeque::with_capacity(c);
            let mut _set = ::std::collections::HashSet::with_capacity(c);
            for _ in 0..c {
                _queue.push_back($value.clone());
                _set.insert($value)
            }

            $crate::data_structures::QueueSet::new(_set, _queue)
        }
    };
}
}
