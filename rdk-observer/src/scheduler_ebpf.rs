use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use aya::{Ebpf, include_bytes_aligned, maps::PerCpuHashMap, programs::TracePoint};
use rdk_observer_core::output_model::SchedulerRecord;
use rdk_observer_core::process_registry::ProcessRegistry;
use tokio::time::MissedTickBehavior;

const EBPF_OBJECT: &[u8] = include_bytes_aligned!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ebpf-test/target/bpfel-unknown-none/release/sched-switch-ebpf"
));

pub async fn run(process_registry: ProcessRegistry) -> Result<()> {
    let mut ebpf = Ebpf::load(EBPF_OBJECT).context("failed to load embedded eBPF object")?;

    {
        let program: &mut TracePoint = ebpf
            .program_mut("count_sched_switch")
            .context("count_sched_switch program was not found")?
            .try_into()
            .context("count_sched_switch is not a tracepoint program")?;

        program.load().context("failed to load eBPF program")?;
        program
            .attach("sched", "sched_switch")
            .context("failed to attach to sched/sched_switch")?;
    }

    let map = ebpf
        .take_map("SWITCH_COUNTS")
        .context("SWITCH_COUNTS map was not found")?;

    let counters = PerCpuHashMap::<_, u64, u64>::try_from(map)
        .context("SWITCH_COUNTS is not a per-CPU hash map")?;

    let mut previous_thread_counts = HashMap::<u64, u64>::new();
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    interval.tick().await;
    let mut sample_started = Instant::now();

    loop {
        interval.tick().await;

        let elapsed = sample_started.elapsed();
        sample_started = Instant::now();

        let mut process_deltas = HashMap::<i32, u64>::new();

        for entry in counters.iter() {
            let (pid_tgid, per_cpu_totals) = entry.context("failed to read SWITCH_COUNTS entry")?;
            let total = per_cpu_totals
                .iter()
                .copied()
                .fold(0_u64, u64::wrapping_add);

            let previous = previous_thread_counts.insert(pid_tgid, total).unwrap_or(0);

            let delta = total.checked_sub(previous).unwrap_or(total);
            let pid = (pid_tgid >> 32) as i32;

            let process_total = process_deltas.entry(pid).or_default();
            *process_total = process_total.wrapping_add(delta);
        }

        let elapsed_seconds = elapsed.as_secs_f64();

        for process in process_registry.get_snapshot() {
            let pid = process.key().pid.0;
            let context_switches = process_deltas.remove(&pid).unwrap_or(0);
            let context_switches_per_second = context_switches as f64 / elapsed_seconds;
            let record = SchedulerRecord {
                context_switches_per_second,
            };

            process_registry.update_process(&process.key(), |process| {
                process.last_scheduler_record = Some(record);
            });
        }
    }
}
