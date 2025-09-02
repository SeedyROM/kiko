use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use kiko::id::{Id, ParticipantId, SessionId, StoryId, VoteId};
use std::collections::HashSet;

fn bench_id_generation_basic(c: &mut Criterion) {
    c.bench_function("session_id_new", |b| b.iter(|| black_box(SessionId::new())));

    c.bench_function("participant_id_new", |b| {
        b.iter(|| black_box(ParticipantId::new()))
    });

    c.bench_function("story_id_new", |b| b.iter(|| black_box(StoryId::new())));

    c.bench_function("vote_id_new", |b| b.iter(|| black_box(VoteId::new())));

    c.bench_function("generic_id_generate", |b| {
        b.iter(|| black_box(Id::<()>::generate()))
    });
}

fn bench_id_generation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_generation_throughput");

    for count in [100, 1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::new("session_id_batch", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let ids: Vec<SessionId> = (0..count).map(|_| SessionId::new()).collect();
                    black_box(ids)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("participant_id_batch", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let ids: Vec<ParticipantId> =
                        (0..count).map(|_| ParticipantId::new()).collect();
                    black_box(ids)
                })
            },
        );
    }

    group.finish();
}

fn bench_id_uniqueness_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_uniqueness");

    for count in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::new("session_id_uniqueness", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut ids = HashSet::new();
                    for _ in 0..count {
                        let id = SessionId::new();
                        ids.insert(id.as_str().to_string());
                    }
                    // All IDs should be unique
                    assert_eq!(ids.len(), count);
                    black_box(ids)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("participant_id_uniqueness", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut ids = HashSet::new();
                    for _ in 0..count {
                        let id = ParticipantId::new();
                        ids.insert(id.as_str().to_string());
                    }
                    // All IDs should be unique
                    assert_eq!(ids.len(), count);
                    black_box(ids)
                })
            },
        );
    }

    group.finish();
}

fn bench_id_operations(c: &mut Criterion) {
    let sample_ids: Vec<SessionId> = (0..1000).map(|_| SessionId::new()).collect();
    let sample_strings: Vec<String> = sample_ids
        .iter()
        .map(|id| id.as_str().to_string())
        .collect();

    c.bench_function("id_as_str", |b| {
        b.iter(|| {
            for id in &sample_ids {
                black_box(id.as_str());
            }
        })
    });

    c.bench_function("id_to_string", |b| {
        b.iter(|| {
            for id in &sample_ids {
                black_box(id.to_string());
            }
        })
    });

    c.bench_function("id_clone", |b| {
        b.iter(|| {
            for id in &sample_ids {
                black_box(id.clone());
            }
        })
    });

    c.bench_function("id_from_string", |b| {
        b.iter(|| {
            for s in &sample_strings {
                black_box(SessionId::from_string(s.clone()));
            }
        })
    });

    c.bench_function("id_from_str_ref", |b| {
        b.iter(|| {
            for s in &sample_strings {
                black_box(SessionId::from(s.as_str()));
            }
        })
    });
}

fn bench_id_custom_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_custom_generation");

    let numeric_alphabet = "0123456789";
    let alpha_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let alphanumeric_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    for &length in [4, 6, 8, 12, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("numeric_custom", length),
            &length,
            |b, &length| b.iter(|| black_box(SessionId::generate_custom(length, numeric_alphabet))),
        );

        group.bench_with_input(
            BenchmarkId::new("alpha_custom", length),
            &length,
            |b, &length| b.iter(|| black_box(SessionId::generate_custom(length, alpha_alphabet))),
        );

        group.bench_with_input(
            BenchmarkId::new("alphanumeric_custom", length),
            &length,
            |b, &length| {
                b.iter(|| black_box(SessionId::generate_custom(length, alphanumeric_alphabet)))
            },
        );
    }

    group.finish();
}

fn bench_id_serialization(c: &mut Criterion) {
    let sample_ids: Vec<SessionId> = (0..1000).map(|_| SessionId::new()).collect();

    c.bench_function("id_serialize_json", |b| {
        b.iter(|| {
            for id in &sample_ids {
                let json = serde_json::to_string(id).unwrap();
                black_box(json);
            }
        })
    });

    let json_strings: Vec<String> = sample_ids
        .iter()
        .map(|id| serde_json::to_string(id).unwrap())
        .collect();

    c.bench_function("id_deserialize_json", |b| {
        b.iter(|| {
            for json in &json_strings {
                let id: SessionId = serde_json::from_str(json).unwrap();
                black_box(id);
            }
        })
    });
}

fn bench_id_hash_collections(c: &mut Criterion) {
    use std::collections::{HashMap, HashSet};

    let sample_ids: Vec<SessionId> = (0..10000).map(|_| SessionId::new()).collect();

    c.bench_function("id_hashset_insert", |b| {
        b.iter(|| {
            let mut set = HashSet::new();
            for id in &sample_ids {
                set.insert(id.clone());
            }
            black_box(set)
        })
    });

    c.bench_function("id_hashmap_insert", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for (i, id) in sample_ids.iter().enumerate() {
                map.insert(id.clone(), i);
            }
            black_box(map)
        })
    });

    let mut pre_populated_set: HashSet<SessionId> = HashSet::new();
    for id in &sample_ids {
        pre_populated_set.insert(id.clone());
    }

    c.bench_function("id_hashset_lookup", |b| {
        b.iter(|| {
            for id in &sample_ids {
                black_box(pre_populated_set.contains(id));
            }
        })
    });

    let mut pre_populated_map: HashMap<SessionId, usize> = HashMap::new();
    for (i, id) in sample_ids.iter().enumerate() {
        pre_populated_map.insert(id.clone(), i);
    }

    c.bench_function("id_hashmap_lookup", |b| {
        b.iter(|| {
            for id in &sample_ids {
                black_box(pre_populated_map.get(id));
            }
        })
    });
}

criterion_group!(
    id_benches,
    bench_id_generation_basic,
    bench_id_generation_throughput,
    bench_id_uniqueness_validation,
    bench_id_operations,
    bench_id_custom_generation,
    bench_id_serialization,
    bench_id_hash_collections
);
criterion_main!(id_benches);
