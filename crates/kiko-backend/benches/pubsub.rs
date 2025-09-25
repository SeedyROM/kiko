use std::{sync::Arc, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

use kiko::{data::SessionMessage, id::SessionId};
use kiko_backend::messaging::PubSub;

fn bench_pubsub_subscribe(c: &mut Criterion) {
    let pubsub = Arc::new(PubSub::new());

    c.bench_function("pubsub_subscribe_single", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let session_id = SessionId::new();
                black_box(pubsub.subscribe(session_id))
            });
    });

    let mut group = c.benchmark_group("pubsub_subscribe_concurrent");
    for concurrent_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*concurrent_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent_count),
            concurrent_count,
            |b, &concurrent_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let futures: Vec<_> = (0..concurrent_count)
                            .map(|_| {
                                let pubsub = pubsub.clone();
                                async move {
                                    let session_id = SessionId::new();
                                    pubsub.subscribe(session_id)
                                }
                            })
                            .collect();

                        black_box(futures::future::join_all(futures).await)
                    });
            },
        );
    }
    group.finish();
}

fn bench_pubsub_publish(c: &mut Criterion) {
    let pubsub = Arc::new(PubSub::new());

    let session_id = SessionId::new();
    let message = SessionMessage::CreateSession(kiko::data::CreateSession {
        name: "Benchmark Session".to_string(),
        duration: Duration::from_secs(3600),
    });

    // Setup subscription first
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _notifier = pubsub.subscribe(session_id.clone());
    });

    c.bench_function("pubsub_publish_single", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                pubsub.publish(session_id.clone(), message.clone());
            });
    });

    let mut group = c.benchmark_group("pubsub_publish_throughput");
    for message_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*message_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(message_count),
            message_count,
            |b, &message_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        for _ in 0..message_count {
                            pubsub.publish(session_id.clone(), message.clone());
                        }
                    });
            },
        );
    }
    group.finish();
}

fn bench_pubsub_consume_events(c: &mut Criterion) {
    let pubsub = Arc::new(PubSub::new());

    let session_id = SessionId::new();
    let message = SessionMessage::CreateSession(kiko::data::CreateSession {
        name: "Benchmark Session".to_string(),
        duration: Duration::from_secs(3600),
    });

    // Setup subscription first
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _notifier = pubsub.subscribe(session_id.clone());
        pubsub.publish(session_id.clone(), message);
    });

    c.bench_function("pubsub_get_event", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async { black_box(pubsub.get_event(&session_id)) });
    });

    let mut group = c.benchmark_group("pubsub_consume_event_cycle");
    for cycle_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*cycle_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(cycle_count),
            cycle_count,
            |b, &cycle_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        for i in 0..cycle_count {
                            let msg = SessionMessage::CreateSession(kiko::data::CreateSession {
                                name: format!("Benchmark Session {i}"),
                                duration: Duration::from_secs(3600),
                            });
                            pubsub.publish(session_id.clone(), msg);
                            black_box(pubsub.consume_event(&session_id));
                        }
                    });
            },
        );
    }
    group.finish();
}

fn bench_pubsub_concurrent_operations(c: &mut Criterion) {
    let pubsub = Arc::new(PubSub::new());

    let mut group = c.benchmark_group("pubsub_concurrent_pub_sub");
    for session_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*session_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(session_count),
            session_count,
            |b, &session_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let futures: Vec<_> = (0..session_count)
                            .map(|i| {
                                let pubsub = pubsub.clone();
                                async move {
                                    let session_id = SessionId::new();
                                    let _notifier = pubsub.subscribe(session_id.clone());

                                    let message =
                                        SessionMessage::CreateSession(kiko::data::CreateSession {
                                            name: format!("Concurrent Session {i}"),
                                            duration: Duration::from_secs(3600),
                                        });

                                    pubsub.publish(session_id.clone(), message);
                                    pubsub.consume_event(&session_id)
                                }
                            })
                            .collect();

                        black_box(futures::future::join_all(futures).await)
                    });
            },
        );
    }
    group.finish();
}

fn bench_pubsub_memory_efficiency(c: &mut Criterion) {
    let pubsub = Arc::new(PubSub::new());

    c.bench_function("pubsub_session_cleanup", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let session_ids: Vec<SessionId> = (0..100).map(|_| SessionId::new()).collect();

                for session_id in &session_ids {
                    let _notifier = pubsub.subscribe(session_id.clone());
                    let message = SessionMessage::CreateSession(kiko::data::CreateSession {
                        name: "Cleanup Test Session".to_string(),
                        duration: Duration::from_secs(3600),
                    });
                    pubsub.publish(session_id.clone(), message);
                }

                for session_id in &session_ids {
                    pubsub.cleanup_session(session_id);
                }
            });
    });
}

criterion_group!(
    pubsub_benches,
    bench_pubsub_subscribe,
    bench_pubsub_publish,
    bench_pubsub_consume_events,
    bench_pubsub_concurrent_operations,
    bench_pubsub_memory_efficiency
);
criterion_main!(pubsub_benches);
