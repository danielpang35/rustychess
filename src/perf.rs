use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy)]
pub struct TimerStat {
    pub total_ns: u64,
    pub count: u64,
}

impl TimerStat {
    pub fn avg_ns(self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.total_ns as f64 / self.count as f64
    }

    pub fn total_ms(self) -> f64 {
        self.total_ns as f64 / 1_000_000.0
    }

    pub fn avg_us(self) -> f64 {
        self.avg_ns() / 1_000.0
    }
}

static EVAL_NEURAL_TIME: AtomicU64 = AtomicU64::new(0);
static EVAL_NEURAL_COUNT: AtomicU64 = AtomicU64::new(0);

static PUSH_TIME: AtomicU64 = AtomicU64::new(0);
static PUSH_COUNT: AtomicU64 = AtomicU64::new(0);

static PUSH_NN_TIME: AtomicU64 = AtomicU64::new(0);
static PUSH_NN_COUNT: AtomicU64 = AtomicU64::new(0);

static POP_TIME: AtomicU64 = AtomicU64::new(0);
static POP_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, Copy)]
pub struct PerfSnapshot {
    pub eval_neural: TimerStat,
    pub push: TimerStat,
    pub push_nn: TimerStat,
    pub pop: TimerStat,
}

fn record(timer: (&AtomicU64, &AtomicU64), d: Duration) {
    let nanos = d.as_nanos() as u64;
    timer.0.fetch_add(nanos, Ordering::Relaxed);
    timer.1.fetch_add(1, Ordering::Relaxed);
}

pub fn record_eval_neural(d: Duration) {
    record((&EVAL_NEURAL_TIME, &EVAL_NEURAL_COUNT), d);
}

pub fn record_push(d: Duration) {
    record((&PUSH_TIME, &PUSH_COUNT), d);
}

pub fn record_push_nn_update(d: Duration) {
    record((&PUSH_NN_TIME, &PUSH_NN_COUNT), d);
}

pub fn record_pop(d: Duration) {
    record((&POP_TIME, &POP_COUNT), d);
}

pub fn reset() {
    for a in [
        &EVAL_NEURAL_TIME,
        &EVAL_NEURAL_COUNT,
        &PUSH_TIME,
        &PUSH_COUNT,
        &PUSH_NN_TIME,
        &PUSH_NN_COUNT,
        &POP_TIME,
        &POP_COUNT,
    ] {
        a.store(0, Ordering::Relaxed);
    }
}

pub fn snapshot() -> PerfSnapshot {
    PerfSnapshot {
        eval_neural: TimerStat {
            total_ns: EVAL_NEURAL_TIME.load(Ordering::Relaxed),
            count: EVAL_NEURAL_COUNT.load(Ordering::Relaxed),
        },
        push: TimerStat {
            total_ns: PUSH_TIME.load(Ordering::Relaxed),
            count: PUSH_COUNT.load(Ordering::Relaxed),
        },
        push_nn: TimerStat {
            total_ns: PUSH_NN_TIME.load(Ordering::Relaxed),
            count: PUSH_NN_COUNT.load(Ordering::Relaxed),
        },
        pop: TimerStat {
            total_ns: POP_TIME.load(Ordering::Relaxed),
            count: POP_COUNT.load(Ordering::Relaxed),
        },
    }
}

pub fn print_snapshot(label: &str, snapshot: PerfSnapshot) {
    println!("{}:", label);
    println!(
        "  eval_neural  total={:.3} ms  count={}  avg={:.3} us",
        snapshot.eval_neural.total_ms(),
        snapshot.eval_neural.count,
        snapshot.eval_neural.avg_us()
    );
    println!(
        "  push         total={:.3} ms  count={}  avg={:.3} us",
        snapshot.push.total_ms(),
        snapshot.push.count,
        snapshot.push.avg_us()
    );
    println!(
        "  push(nn)     total={:.3} ms  count={}  avg={:.3} us",
        snapshot.push_nn.total_ms(),
        snapshot.push_nn.count,
        snapshot.push_nn.avg_us()
    );
    println!(
        "  pop          total={:.3} ms  count={}  avg={:.3} us",
        snapshot.pop.total_ms(),
        snapshot.pop.count,
        snapshot.pop.avg_us()
    );
}
