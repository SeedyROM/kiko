use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

use kiko::{data::CreateSession, id::SessionId};
use kiko_backend::services::{SessionService, SessionServiceInMemory};

fn bench_session_create(c: &mut Criterion) {
    let service = SessionServiceInMemory::new();

    c.bench_function("session_create_single", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let service = service.clone();
                async move {
                    let create_request = CreateSession {
                        name: "Benchmark Session".to_string(),
                        duration: Duration::from_secs(3600),
                    };
                    black_box(service.create(create_request).await.unwrap())
                }
            });
    });

    let mut group = c.benchmark_group("session_create_throughput");
    for session_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*session_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(session_count),
            session_count,
            |b, &session_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let service = service.clone();
                        async move {
                            for i in 0..session_count {
                                let create_request = CreateSession {
                                    name: format!("Benchmark Session {i}"),
                                    duration: Duration::from_secs(3600),
                                };
                                black_box(service.create(create_request).await.unwrap());
                            }
                        }
                    });
            },
        );
    }
    group.finish();
}

fn bench_session_get(c: &mut Criterion) {
    let service = SessionServiceInMemory::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let session_ids: Vec<SessionId> = rt.block_on(async {
        let mut ids = Vec::new();
        for i in 0..1000 {
            let create_request = CreateSession {
                name: format!("Get Benchmark Session {i}"),
                duration: Duration::from_secs(3600),
            };
            let session = service.create(create_request).await.unwrap();
            ids.push(session.id);
        }
        ids
    });

    c.bench_function("session_get_single", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let service = service.clone();
                let session_id = session_ids[0].clone();
                async move { black_box(service.get(&session_id).await.unwrap()) }
            });
    });

    let mut group = c.benchmark_group("session_get_random_access");
    for access_count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*access_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(access_count),
            access_count,
            |b, &access_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let service = service.clone();
                        let session_ids = session_ids.clone();
                        async move {
                            use rand::seq::SliceRandom;
                            let mut rng = rand::thread_rng();
                            for _ in 0..access_count {
                                let session_id = session_ids.choose(&mut rng).unwrap();
                                black_box(service.get(session_id).await.unwrap());
                            }
                        }
                    });
            },
        );
    }
    group.finish();
}

fn bench_session_list(c: &mut Criterion) {
    let service = SessionServiceInMemory::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for i in 0..1000 {
            let create_request = CreateSession {
                name: format!("List Benchmark Session {i}"),
                duration: Duration::from_secs(3600),
            };
            let _session = service.create(create_request).await.unwrap();
        }
    });

    c.bench_function("session_list_1000_sessions", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let service = service.clone();
                async move { black_box(service.list().await.unwrap()) }
            });
    });
}

fn bench_session_participant_operations(c: &mut Criterion) {
    let service = SessionServiceInMemory::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let session_id = rt.block_on(async {
        let create_request = CreateSession {
            name: "Participant Benchmark Session".to_string(),
            duration: Duration::from_secs(3600),
        };
        let session = service.create(create_request).await.unwrap();
        session.id
    });

    c.bench_function("session_join_participant", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let service = service.clone();
                let session_id = session_id.clone();
                async move {
                    let participant_name = format!("Participant_{}", rand::random::<u32>());
                    black_box(service.join(&session_id, &participant_name).await.unwrap())
                }
            });
    });

    let mut group = c.benchmark_group("session_participant_join_leave_cycle");
    for participant_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*participant_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(participant_count),
            participant_count,
            |b, &participant_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let service = service.clone();
                        let session_id = session_id.clone();
                        async move {
                            let mut participant_ids = Vec::new();

                            // Join participants
                            for i in 0..participant_count {
                                let participant_name = format!("Cycle_Participant_{i}");
                                let participant =
                                    service.join(&session_id, &participant_name).await.unwrap();
                                participant_ids.push(participant.id().clone());
                            }

                            // Leave participants
                            for participant_id in participant_ids {
                                service.leave(&session_id, &participant_id).await.unwrap();
                            }
                        }
                    });
            },
        );
    }
    group.finish();
}

fn bench_session_concurrent_access(c: &mut Criterion) {
    let service = SessionServiceInMemory::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let session_id = rt.block_on(async {
        let create_request = CreateSession {
            name: "Concurrent Access Session".to_string(),
            duration: Duration::from_secs(3600),
        };
        let session = service.create(create_request).await.unwrap();
        session.id
    });

    let mut group = c.benchmark_group("session_concurrent_operations");
    for concurrent_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*concurrent_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent_count),
            concurrent_count,
            |b, &concurrent_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let service = service.clone();
                        let session_id = session_id.clone();
                        async move {
                            let futures: Vec<_> = (0..concurrent_count)
                                .map(|i| {
                                    let service = service.clone();
                                    let session_id = session_id.clone();
                                    async move {
                                        let participant_name =
                                            format!("Concurrent_Participant_{i}");
                                        let participant = service
                                            .join(&session_id, &participant_name)
                                            .await
                                            .unwrap();
                                        let _session = service.get(&session_id).await.unwrap();
                                        service.leave(&session_id, participant.id()).await.unwrap();
                                    }
                                })
                                .collect();

                            black_box(futures::future::join_all(futures).await)
                        }
                    });
            },
        );
    }
    group.finish();
}

fn bench_session_update_performance(c: &mut Criterion) {
    let service = SessionServiceInMemory::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let session_id = rt.block_on(async {
        let create_request = CreateSession {
            name: "Update Performance Session".to_string(),
            duration: Duration::from_secs(3600),
        };
        let session = service.create(create_request).await.unwrap();
        session.id
    });

    c.bench_function("session_update_single", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let service = service.clone();
                let session_id = session_id.clone();
                async move {
                    let mut session = service.get(&session_id).await.unwrap();
                    session.set_topic(format!("Topic {}", rand::random::<u32>()));
                    black_box(service.update(&session_id, &session).await.unwrap())
                }
            });
    });

    let mut group = c.benchmark_group("session_update_with_participants");
    for participant_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*participant_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(participant_count),
            participant_count,
            |b, &participant_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let service = service.clone();
                        async move {
                            // Create fresh session with participants
                            let create_request = CreateSession {
                                name: "Update with Participants Session".to_string(),
                                duration: Duration::from_secs(3600),
                            };
                            let mut session = service.create(create_request).await.unwrap();
                            let session_id = session.id.clone();

                            // Add participants
                            for i in 0..participant_count {
                                let participant_name = format!("Update_Participant_{i}");
                                let _participant =
                                    service.join(&session_id, &participant_name).await.unwrap();
                            }

                            // Get updated session and modify
                            session = service.get(&session_id).await.unwrap();
                            session.set_topic("Updated Topic".to_string());
                            black_box(service.update(&session_id, &session).await.unwrap());

                            // Cleanup
                            service.end(&session_id).await.unwrap();
                        }
                    });
            },
        );
    }
    group.finish();
}

criterion_group!(
    session_service_benches,
    bench_session_create,
    bench_session_get,
    bench_session_list,
    bench_session_participant_operations,
    bench_session_concurrent_access,
    bench_session_update_performance
);
criterion_main!(session_service_benches);
