use anyhow::{Context, Result};
use std::{collections::HashMap, fmt::Display, fs, path::Path};

use super::common;

pub trait StatsProvider {
    type Stats;

    fn stats(cgroup_path: &Path) -> Result<Self::Stats>;
}

/// Reports the statistics for a cgroup
#[derive(Debug)]
pub struct Stats {
    /// Cpu statistics for the cgroup
    pub cpu: CpuStats,
    /// Pid statistics for the cgroup
    pub pids: PidStats,
    /// Hugetlb statistics for the cgroup
    pub hugetlb: HashMap<String, HugeTlbStats>,
    /// Blkio statistics for the cgroup
    pub blkio: BlkioStats,
    /// Memory statistics for the cgroup
    pub memory: MemoryStats,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            cpu: CpuStats::default(),
            pids: PidStats::default(),
            hugetlb: HashMap::new(),
            blkio: BlkioStats::default(),
            memory: MemoryStats::default(),
        }
    }
}

/// Reports the cpu statistics for a cgroup
#[derive(Debug)]
pub struct CpuStats {
    /// Cpu usage statistics for the cgroup
    pub usage: CpuUsage,
    /// Cpu Throttling statistics for the cgroup
    pub throttling: CpuThrottling,
}

impl Default for CpuStats {
    fn default() -> Self {
        Self {
            usage: CpuUsage::default(),
            throttling: CpuThrottling::default(),
        }
    }
}

/// Reports the cpu usage for a cgroup
#[derive(Debug, PartialEq, Eq)]
pub struct CpuUsage {
    /// Cpu time consumed by tasks in total
    pub usage_total: u64,
    /// Cpu time consumed by tasks in user mode
    pub usage_user: u64,
    /// Cpu time consumed by tasks in kernel mode
    pub usage_kernel: u64,
    /// Cpu time consumed by tasks itemized per core
    pub per_core_usage_total: Vec<u64>,
    /// Cpu time consumed by tasks in user mode itemized per core
    pub per_core_usage_user: Vec<u64>,
    /// Cpu time consumed by tasks in kernel mode itemized per core
    pub per_core_usage_kernel: Vec<u64>,
}

impl Default for CpuUsage {
    fn default() -> Self {
        Self {
            usage_total: 0,
            usage_user: 0,
            usage_kernel: 0,
            per_core_usage_total: Vec::new(),
            per_core_usage_user: Vec::new(),
            per_core_usage_kernel: Vec::new(),
        }
    }
}

/// Reports the cpu throttling for a cgroup
#[derive(Debug, PartialEq, Eq)]
pub struct CpuThrottling {
    /// Number of period intervals (as specified in cpu.cfs_period_us) that have elapsed
    pub periods: u64,
    /// Number of period intervals where tasks have been throttled because they exhausted their quota
    pub throttled_periods: u64,
    /// Total time duration for which tasks have been throttled
    pub throttled_time: u64,
}

impl Default for CpuThrottling {
    fn default() -> Self {
        Self {
            periods: 0,
            throttled_periods: 0,
            throttled_time: 0,
        }
    }
}

/// Reports memory stats for a cgroup
#[derive(Debug)]
pub struct MemoryStats {
    /// Usage of memory
    pub memory: MemoryData,
    /// Usage of memory and swap
    pub memswap: MemoryData,
    /// Usage of kernel memory
    pub kernel: MemoryData,
    /// Usage of kernel tcp memory
    pub kernel_tcp: MemoryData,
    /// Page cache in bytes
    pub cache: u64,
    /// Returns true if hierarchical accounting is enabled
    pub hierarchy: bool,
    /// Various memory statistics
    pub stats: HashMap<String, u64>,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            memory: MemoryData::default(),
            memswap: MemoryData::default(),
            kernel: MemoryData::default(),
            kernel_tcp: MemoryData::default(),
            cache: 0,
            hierarchy: false,
            stats: HashMap::default(),
        }
    }
}

/// Reports memory stats for one type of memory
#[derive(Debug, PartialEq, Eq)]
pub struct MemoryData {
    /// Usage in bytes
    pub usage: u64,
    /// Maximum recorded usage in bytes
    pub max_usage: u64,
    /// Number of times memory usage hit limits
    pub fail_count: u64,
    /// Memory usage limit
    pub limit: u64,
}

impl Default for MemoryData {
    fn default() -> Self {
        Self {
            usage: 0,
            max_usage: 0,
            fail_count: 0,
            limit: 0,
        }
    }
}

/// Reports pid stats for a cgroup
#[derive(Debug, PartialEq, Eq)]
pub struct PidStats {
    /// Current number of active pids
    pub current: u64,
    /// Allowed number of active pids (0 means no limit)
    pub limit: u64,
}

impl Default for PidStats {
    fn default() -> Self {
        Self {
            current: 0,
            limit: 0,
        }
    }
}

/// Reports block io stats for a cgroup
#[derive(Debug, PartialEq, Eq)]
pub struct BlkioStats {
    // Number of bytes transfered to/from a device by the cgroup
    pub service_bytes: Vec<BlkioDeviceStat>,
    // Number of I/O operations performed on a device by the cgroup
    pub serviced: Vec<BlkioDeviceStat>,
    // Time in milliseconds that the cgroup had access to a device
    pub time: Vec<BlkioDeviceStat>,
    // Number of sectors transferred to/from a device by the cgroup
    pub sectors: Vec<BlkioDeviceStat>,
    // Total time between request dispatch and request completion
    pub service_time: Vec<BlkioDeviceStat>,
    // Total time spend waiting in the scheduler queues for service
    pub wait_time: Vec<BlkioDeviceStat>,
    // Number of requests queued for I/O operations
    pub queued: Vec<BlkioDeviceStat>,
    // Number of requests merged into requests for I/O operations
    pub merged: Vec<BlkioDeviceStat>,
}

impl Default for BlkioStats {
    fn default() -> Self {
        Self {
            service_bytes: Vec::new(),
            serviced: Vec::new(),
            time: Vec::new(),
            sectors: Vec::new(),
            service_time: Vec::new(),
            wait_time: Vec::new(),
            queued: Vec::new(),
            merged: Vec::new(),
        }
    }
}

/// Reports single stat value for a specific device
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlkioDeviceStat {
    /// Major device number
    pub major: u64,
    /// Minor device number
    pub minor: u64,
    /// Operation type
    pub op_type: Option<String>,
    /// Stat value
    pub value: u64,
}

impl Display for BlkioDeviceStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(op_type) = &self.op_type {
            write!(
                f,
                "{}:{} {} {}",
                self.major, self.minor, op_type, self.value
            )
        } else {
            write!(f, "{}:{} {}", self.major, self.minor, self.value)
        }
    }
}

/// Reports hugetlb stats for a cgroup
#[derive(Debug, PartialEq, Eq)]
pub struct HugeTlbStats {
    /// Current usage in bytes
    pub usage: u64,
    /// Maximum recorded usage in bytes
    pub max_usage: u64,
    /// Number of allocation failures due to HugeTlb usage limit
    pub fail_count: u64,
}

impl Default for HugeTlbStats {
    fn default() -> Self {
        Self {
            usage: 0,
            max_usage: 0,
            fail_count: 0,
        }
    }
}

pub fn supported_page_sizes() -> Result<Vec<String>> {
    let mut sizes = Vec::new();
    for hugetlb_entry in fs::read_dir("/sys/kernel/mm/hugepages")? {
        let hugetlb_entry = hugetlb_entry?;
        if !hugetlb_entry.path().is_dir() {
            continue;
        }

        let file_name = hugetlb_entry.file_name();
        let file_name = file_name.to_str().unwrap();
        if let Some(name_stripped) = file_name.strip_prefix("hugepages-") {
            if let Some(size) = name_stripped.strip_suffix("kB") {
                let size: u64 = size.parse()?;

                let size_moniker = if size >= (1 << 20) {
                    (size >> 20).to_string() + "GB"
                } else if size >= (1 << 10) {
                    (size >> 10).to_string() + "MB"
                } else {
                    size.to_string() + "KB"
                };

                sizes.push(size_moniker);
            }
        }
    }

    Ok(sizes)
}

pub fn parse_single_value(file_path: &Path) -> Result<u64> {
    let value = common::read_cgroup_file(file_path)?;
    value.trim().parse().with_context(|| {
        format!(
            "failed to parse value {} from {}",
            value,
            file_path.display()
        )
    })
}

pub fn pid_stats(cgroup_path: &Path) -> Result<PidStats> {
    let mut stats = PidStats::default();

    let current = common::read_cgroup_file(cgroup_path.join("pids.current"))?;
    stats.current = current
        .trim()
        .parse()
        .context("failed to parse current pids")?;

    let limit =
        common::read_cgroup_file(cgroup_path.join("pids.max")).map(|l| l.trim().to_owned())?;
    if limit != "max" {
        stats.limit = limit.parse().context("failed to parse pids limit")?;
    }

    Ok(stats)
}
