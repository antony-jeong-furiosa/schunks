#[cfg(test)]
mod tests_for_schunks {

    #[allow(unused)]
    fn is_type_send<T: Send>() {}
    #[allow(unused)]
    fn is_type_sync<T: Sync>() {}
    #[allow(unused)]
    fn is_type_send_sync<T: Send + Sync>() {}
    #[allow(unused)]
    fn is_send<T: Send>(v: T) {}
    #[allow(unused)]
    fn is_sync<T: Sync>(v: T) {}
    #[allow(unused)]
    fn is_send_sync<T: Send + Sync>(v: T) {}
    #[allow(unused)]
    fn is_sendable_iter<T: Iterator + Send>(v: T)
        where T::Item: Send
    {}

    // To get around compiler optimization
    #[allow(unused)]
    fn collatz(mut n: usize) -> usize {
        const STREAK_GOAL: usize = 128;
        let mut streak = 0;
        let mut prev1 = usize::MAX;
        let mut prev2 = usize::MAX;
        let mut prev3;
        loop {
            prev3 = prev2;
            prev2 = prev1;
            prev1 = n;
            if n % 2 == 0 {
                n /= 2;
            } else {
                n = 3*n + 1;
            }
            if n == prev3 {
                streak += 1;
            } else {
                streak = 0;
            }
            if streak == STREAK_GOAL {
                return prev1.min(prev2).min(prev3);
            }
        }
    }

    #[test]
    fn channel_par_schunks() {
        use schunks::*;
        use itertools::Itertools;
        use std::sync::mpsc::channel;
        const MAX: usize = 256;
        const CHUNK_SIZE: usize = 16;

        let rx = {
            let (tx, rx) = channel();
            for i in 0..MAX {
                tx.send(i).unwrap();
            }
            rx
        };
        let mut collected_par = rx.into_iter()
            .par_schunks(CHUNK_SIZE)
            .map(|c| c.collect::<Vec<_>>())
            .collect::<Vec<_>>();
        collected_par.sort_unstable();

        let c = (0..MAX).chunks(CHUNK_SIZE);
        let collected_ser = c.into_iter()
            .map(|c| c.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        assert_eq!(collected_par, collected_ser);
    }

    #[test]
    fn zip_par_schunks() {
        use schunks::*;
        use std::sync::Mutex;
        const MAX: usize = 256;
        const CHUNK_SIZE: usize = 16;

        let input = (0..MAX).collect::<Vec<_>>();
        let mut output = Vec::new();
        for _ in 0..MAX {
            output.push(Mutex::new(0));
        }
        let zipped = input.iter().zip(output.iter());
        zipped.par_schunks(CHUNK_SIZE).for_each(|c| {
            c.for_each(|(e, m)| {
                let mut r = m.lock().unwrap();
                *r = e*2;
            })
        });
        for i in 0..MAX {
            assert_eq!(input[i]*2, *output[i].lock().unwrap())
        }
    }

    #[test]
    fn keeps_limit() {
        use schunks::*;
        use rayon::ThreadPoolBuilder;
        use std::sync::{Mutex, atomic::{AtomicUsize, Ordering::SeqCst}};
        const MAX: usize = 1023;
        const CHUNK_SIZE: usize = 16;
        const NUM_THREADS: usize = 16;
        const COLLATZ_MULT: usize = 65536;

        struct It<'a, I: Iterator> {
            inner: I,
            num_working: &'a AtomicUsize,
            max_num_working: &'a AtomicUsize,
            index: AtomicUsize,
        }
        impl<'a, I> Iterator for It<'a, I>
            where I: Iterator
        {
            type Item = I::Item;
            fn next(&mut self) -> Option<Self::Item> {
                let curr_index = self.index.fetch_add(1, SeqCst);
                if curr_index % CHUNK_SIZE == 0 && curr_index < MAX {
                    let curr_working = self.num_working.fetch_add(1, SeqCst) + 1;
                    let old_max = self.max_num_working.load(SeqCst);
                    if curr_working > old_max {
                        let _ = self.max_num_working.compare_exchange(old_max, curr_working, SeqCst, SeqCst);
                    }
                    assert!(curr_working <= NUM_THREADS);
                }
                self.inner.next()
            }
        }

        let num_working = AtomicUsize::new(0);
        let max_num_working = AtomicUsize::new(0);
        let input_vec = (0..MAX).collect::<Vec<_>>();
        let input = It {
            inner: 0..MAX,
            num_working: &num_working,
            max_num_working: &max_num_working,
            index: AtomicUsize::new(0),
        };
        let output = (0..MAX).map(|_| Mutex::new(0 as usize)).collect::<Vec<_>>();
        let zipped = input.zip(output.iter());
        let pool = ThreadPoolBuilder::new().num_threads(NUM_THREADS).build().unwrap();
        pool.install(|| {
            zipped.par_schunks(CHUNK_SIZE).for_each(|c| {
                c.for_each(|(e, m)| {
                    // To make the operation expensive
                    let seed = e + 1;
                    let mut v = Vec::new();
                    for i in 1..COLLATZ_MULT {
                        v.push(collatz(i*seed));
                    }
                    let mean = v.iter().sum::<usize>() / (COLLATZ_MULT-1);
                    //

                    let mut r = m.lock().unwrap();
                    *r = e * mean * 2;
                });
                let _old = num_working.fetch_sub(1, SeqCst);
            });
        });
        for i in 0..MAX {
            assert_eq!(input_vec[i]*2, *output[i].lock().unwrap())
        }
    }

    // #[test]
    // fn collatz_works() {
    //     const SEED: usize = 515253;
    //     const MULT: usize = 255123;
    //     let mut v = Vec::new();
    //     for i in 1..MULT {
    //         v.push(collatz(i*SEED));
    //     }
    //     for r in v {
    //         assert_eq!(r, 1);
    //     }
    // }
}
