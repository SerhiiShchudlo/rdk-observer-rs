# rdk-observer-rs

`rdk-observer-rs` is a system process observer for RDK devices. Its goal is to inspect running processes, create records from the observations, and send those records to a backend for analysis and monitoring.

## Problem solved

It provides continuous, low-overhead observability of system resources across an RDK device, including CPU, DRAM, processes, network, and I/O activity.

In a resource-constrained device, the main goal is to understand **where the available resources are going** and which processes or kernel activities are consuming them. By combining system metrics, process-level information, and Linux observability mechanisms, the component helps track resource usage over time, correlate transient events with their likely cause, and provide the data needed to evaluate the efficiency of current components and support future architectural decisions.

## Planned tracking options

The observer is intended to support multiple ways to choose what to track:

- All user-space processes.
- Kernel processes and threads, where the system exposes them.
- Processes associated with RDK-B.
- Individually selected processes.

These options will allow deployments to choose between a broad system view and focused monitoring of specific processes.

This repository is currently a project scaffold; the observer and its tracking options are planned functionality.

## Functional requirements

All resource monitors must operate at per-process granularity and attribute observations to the stable process identity (`PID + start_time`) whenever the Linux kernel exposes the required data. System-wide metrics must also be collected to provide context. Metrics that cannot be attributed reliably to an individual process must remain identified as system-wide rather than being assigned inaccurately.

### Process registry

- Track running processes and their lifecycle.
- Maintain stable identity using `PID + start_time`.
- Collect metadata such as name, executable, parent PID, and command line.
- Keep current and recent historical process information.
- Provide a shared process registry to the other monitors.
- Minimize discovery overhead.

### CPU resource management

- Monitor overall CPU utilization.
- Track CPU usage per process.
- Measure load and CPU pressure.
- Track context switches.
- Collect relevant perf/PMU counters.
- Detect short CPU peaks and unusual CPU activity.

### DRAM resource management

- Monitor total, available, and used DRAM.
- Track memory usage per process using RSS, PSS, and private memory.
- Monitor page cache and kernel slab usage.
- Track memory pressure using PSI.
- Monitor page faults, reclaim, swap, and other memory activity.
- Detect memory growth, peaks, and potential OOM conditions.

### Storage resource monitor

- Monitor system storage read/write activity.
- Track I/O usage per process.
- Monitor I/O operations, throughput, and latency where available.
- Track I/O pressure using PSI.
- Detect processes generating unusually high storage activity.

### Network resource monitor

- Monitor traffic per network interface.
- Track network resource usage per process.
- Track RX/TX bytes and packets.
- Monitor packet drops and errors.
- Track interface state and link changes.
- Collect relevant network-stack statistics.
- Detect unusual traffic or network resource consumption.

### Aggregator and metrics manager

- Receive metrics from all monitoring workers.
- Correlate CPU, DRAM, network, and I/O information with processes.
- Maintain short-term historical data.
- Calculate averages, peaks, deltas, and trends.
- Reduce high-frequency samples into compact telemetry.
- Control the frequency at which metrics are exported.
- Track the overhead introduced by the component itself.

### Collector (external)

- Collect metrics from RDK-Observer and store them.
- Provide dashboards and initial analysis.
- Potentially provide input to an AI model for deeper analysis.

## Deliverables

RDK-Observer should have clear non-functional deliverables, including maximum storage footprint, CPU and DRAM usage, and portability across RDK-B platforms. The component can also run inside a dedicated **cgroup**, allowing resource limits to be enforced and validated during operation.

### Initial resource targets

The initial resource targets for RDK-Observer are approximately **5 MB maximum storage footprint**, **10 MB maximum DRAM usage**, and **below 5% CPU utilization** under normal operation. The component will run in a dedicated **cgroup** so these resource limits can be measured and, where appropriate, enforced.

## Architecture decisions

- [ADR-0001: Collect System Resource Snapshots from procfs](docs/architecture/adr/0001-collect-system-resource-snapshots-from-procfs.md)
- [ADR-0002: Schedule Per-Process Sampling Workers](docs/architecture/adr/0002-schedule-per-process-sampling-workers.md)
- [ADR-0003: Trigger Process Capture with Aya and BPF Events](docs/architecture/adr/0003-trigger-process-capture-with-bpf-events.md)
- [ADR-0004: Use CBOR for Observation Records](docs/architecture/adr/0004-use-cbor-for-observation-records.md)
