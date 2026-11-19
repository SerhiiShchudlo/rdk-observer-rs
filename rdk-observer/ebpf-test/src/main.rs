#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::bpf_get_current_pid_tgid,
    macros::{map, tracepoint},
    maps::LruPerCpuHashMap,
    programs::TracePointContext,
};

#[map]
static SWITCH_COUNTS: LruPerCpuHashMap<u64, u64> =
    LruPerCpuHashMap::with_max_entries(4096, 0);

#[tracepoint]
pub fn count_sched_switch(_ctx: TracePointContext) -> u32 {
    let pid_tgid = bpf_get_current_pid_tgid();

    if pid_tgid == 0 {
        return 0;
    }

    if let Some(counter) = SWITCH_COUNTS.get_ptr_mut(&pid_tgid) {
        unsafe {
            *counter = (*counter).wrapping_add(1);
        }
    } else {
        let _ = SWITCH_COUNTS.insert(&pid_tgid, &1_u64, 0);
    }

    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
