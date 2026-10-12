use std::{
    collections::HashSet,
    fs,
    sync::{Arc, Barrier},
};

#[path = "support/temporary.rs"]
mod temporary_support;

#[test]
fn concurrent_fixtures_keep_independent_contents_and_cleanup() {
    let barrier = Arc::new(Barrier::new(32));
    let workers = (0..32)
        .map(|worker| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                (0..100)
                    .map(|allocation| {
                        let root = temporary_support::temporary_directory("seseragi-isolation");
                        let contents = format!("{worker}:{allocation}");
                        fs::write(root.join("owner"), &contents).unwrap();
                        (root, contents)
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    let roots = workers
        .into_iter()
        .flat_map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        roots
            .iter()
            .map(|(root, _)| root)
            .collect::<HashSet<_>>()
            .len(),
        roots.len()
    );
    for (root, expected) in &roots {
        assert_eq!(fs::read_to_string(root.join("owner")).unwrap(), *expected);
        fs::remove_dir_all(root).unwrap();
    }
}
