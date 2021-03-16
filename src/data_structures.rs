use std::cmp::Eq;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

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
        if let None = self.set.get(&item) {
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
                _set.insert($value)
            )*;

            QueueSet::new(_set, _queue)
        }
    };
    ($value:expr;$count:expr) => {
        {
            let c = $count;
            let mut _queue = ::std::collections::VecDeque::with_capacity(c);
            let mut _set = ::std::collections::HashSet::with_capacity(_cap);
            for _ in 0..c {
                _queue.push_back($value.clone());
                _set.insert($value)
            }

            QueueSet::new(_set, _queue)
        }
    };
}
}
