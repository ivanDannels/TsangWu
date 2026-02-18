use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

use crate::persistence::WorkflowPersistence;

/// 工作流调度器：负责超时检测和重试调度
pub struct WorkflowScheduler {
    check_interval: Duration,
    persistence: Arc<dyn WorkflowPersistence>,
}

impl WorkflowScheduler {
    pub fn new(check_interval: Duration, persistence: Arc<dyn WorkflowPersistence>) -> Self {
        Self {
            check_interval,
            persistence,
        }
    }

    /// 启动调度循环（后台任务）
    pub async fn start(&self) {
        let mut ticker = interval(self.check_interval);
        loop {
            ticker.tick().await;
            self.scan_timeouts().await;
            self.scan_stale_workflows().await;
        }
    }

    /// 扫描超时的工作流步骤
    async fn scan_timeouts(&self) {
        tracing::debug!("扫描超时工作流步骤...");
        // 实际实现需要查询 DB 中长时间处于 running 状态的实例
        let _ = &self.persistence;
    }

    /// 扫描停滞的工作流
    async fn scan_stale_workflows(&self) {
        tracing::debug!("扫描停滞工作流...");
        // 实际实现需要查询 DB 中长时间未更新的实例
        let _ = &self.persistence;
    }
}
