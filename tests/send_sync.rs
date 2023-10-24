mod for_itertools {
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

    #[test]
    fn channel_par_schunks() {
        use rayon::prelude::*;
        use schunks::SchunksTools;
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
        let mut collected_par = rx.into_iter().par_schunks(CHUNK_SIZE).map(|c| c.collect::<Vec<_>>()).collect::<Vec<_>>();
        collected_par.sort_unstable();

        let c = (0..MAX).chunks(CHUNK_SIZE);
        let collected_ser = c.into_iter().map(|c| c.collect::<Vec<_>>()).collect::<Vec<_>>();

        assert_eq!(collected_par, collected_ser);
    }

    #[test]
    fn zip_par_schunks() {
        use schunks::SchunksTools;
        use rayon::prelude::*;
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
}