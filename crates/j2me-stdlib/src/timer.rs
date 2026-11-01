use j2me_core::heap::object::ObjectId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskScheduleType {
    OneShot,
    FixedDelay,
    FixedRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Virgin,
    Scheduled,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TimerTaskEntry {
    pub task_id: ObjectId,
    pub next_execution_time: u64,
    pub period: u64,
    pub schedule_type: TaskScheduleType,
    pub state: TaskState,
    pub last_scheduled_time: u64,
}

#[derive(Debug, Clone)]
pub struct TimerEntry {
    pub timer_id: ObjectId,
    pub tasks: Vec<TimerTaskEntry>,
    pub cancelled: bool,
}

#[derive(Debug, Default)]
pub struct TimerManager {
    timers: HashMap<ObjectId, TimerEntry>,
    task_to_timer: HashMap<ObjectId, ObjectId>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_tasks_count(&self) -> usize {
        self.timers
            .values()
            .map(|t| {
                t.tasks
                    .iter()
                    .filter(|tk| tk.state == TaskState::Scheduled)
                    .count()
            })
            .sum()
    }

    pub fn schedule(
        &mut self,
        timer_id: ObjectId,
        task_id: ObjectId,
        delay_ms: u64,
        period_ms: Option<u64>,
        is_fixed_rate: bool,
        current_time_ms: u64,
    ) -> Result<(), &'static str> {
        // A task cannot be scheduled more than once or if already cancelled
        if let Some(existing_timer_id) = self.task_to_timer.get(&task_id) {
            if let Some(existing_timer) = self.timers.get(existing_timer_id) {
                if let Some(t) = existing_timer.tasks.iter().find(|t| t.task_id == task_id) {
                    if t.state != TaskState::Virgin {
                        return Err("IllegalStateException: Task already scheduled or cancelled");
                    }
                }
            }
        }

        let timer = self.timers.entry(timer_id).or_insert_with(|| TimerEntry {
            timer_id,
            tasks: Vec::new(),
            cancelled: false,
        });

        if timer.cancelled {
            return Err("IllegalStateException: Timer was cancelled");
        }

        let schedule_type = match period_ms {
            None => TaskScheduleType::OneShot,
            Some(_) if is_fixed_rate => TaskScheduleType::FixedRate,
            Some(_) => TaskScheduleType::FixedDelay,
        };

        let next_execution = current_time_ms.saturating_add(delay_ms);
        let period = period_ms.unwrap_or(0);

        let entry = TimerTaskEntry {
            task_id,
            next_execution_time: next_execution,
            period,
            schedule_type,
            state: TaskState::Scheduled,
            last_scheduled_time: next_execution,
        };

        timer.tasks.push(entry);
        self.task_to_timer.insert(task_id, timer_id);
        Ok(())
    }

    pub fn cancel_timer(&mut self, timer_id: ObjectId) {
        if let Some(timer) = self.timers.get_mut(&timer_id) {
            timer.cancelled = true;
            for t in &mut timer.tasks {
                t.state = TaskState::Cancelled;
            }
        }
    }

    pub fn cancel_task(&mut self, task_id: ObjectId) -> bool {
        if let Some(timer_id) = self.task_to_timer.get(&task_id) {
            if let Some(timer) = self.timers.get_mut(timer_id) {
                if let Some(t) = timer.tasks.iter_mut().find(|t| t.task_id == task_id) {
                    if t.state == TaskState::Scheduled {
                        t.state = TaskState::Cancelled;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn scheduled_execution_time(&self, task_id: ObjectId) -> Option<u64> {
        if let Some(timer_id) = self.task_to_timer.get(&task_id) {
            if let Some(timer) = self.timers.get(timer_id) {
                if let Some(t) = timer.tasks.iter().find(|t| t.task_id == task_id) {
                    return Some(t.last_scheduled_time);
                }
            }
        }
        None
    }

    /// Ticks timers and collects tasks ready to execute at current_time_ms.
    /// Sequential per timer: tasks of a single Timer run sequentially.
    pub fn advance(&mut self, current_time_ms: u64) -> Vec<ObjectId> {
        let mut ready = Vec::new();

        for timer in self.timers.values_mut() {
            if timer.cancelled {
                continue;
            }
            for task in &mut timer.tasks {
                if task.state != TaskState::Scheduled {
                    continue;
                }
                if current_time_ms >= task.next_execution_time {
                    task.last_scheduled_time = task.next_execution_time;
                    ready.push(task.task_id);

                    match task.schedule_type {
                        TaskScheduleType::OneShot => {
                            task.state = TaskState::Executed;
                        }
                        TaskScheduleType::FixedDelay => {
                            // Fixed-delay: next execution is relative to actual execution start
                            task.next_execution_time = current_time_ms.saturating_add(task.period);
                        }
                        TaskScheduleType::FixedRate => {
                            // Fixed-rate: next execution is relative to initial scheduled time
                            task.next_execution_time =
                                task.next_execution_time.saturating_add(task.period);
                            // Avoid infinite overdue catch-up storms if paused for too long
                            if task.next_execution_time + task.period * 10 < current_time_ms {
                                task.next_execution_time =
                                    current_time_ms.saturating_add(task.period);
                            }
                        }
                    }
                }
            }
        }

        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_task_executes_once() {
        let mut mgr = TimerManager::new();
        let timer = ObjectId(1);
        let task = ObjectId(10);

        mgr.schedule(timer, task, 50, None, false, 100).unwrap();
        assert_eq!(mgr.advance(120), vec![]);
        assert_eq!(mgr.advance(150), vec![task]);
        assert_eq!(mgr.advance(200), vec![]); // does not run again
    }

    #[test]
    fn fixed_delay_vs_fixed_rate() {
        let mut mgr = TimerManager::new();
        let timer = ObjectId(1);
        let task_delay = ObjectId(11);
        let task_rate = ObjectId(12);

        // Schedule at t=0, delay=10, period=20
        mgr.schedule(timer, task_delay, 10, Some(20), false, 0)
            .unwrap();
        mgr.schedule(timer, task_rate, 10, Some(20), true, 0)
            .unwrap();

        // First firing at t=10
        let r1 = mgr.advance(10);
        assert!(r1.contains(&task_delay) && r1.contains(&task_rate));

        // Advance to t=40 (delayed by 10ms past the normal 30ms mark)
        // Fixed-delay scheduled next at 10 + 20 = 30? No, relative to firing t=10 + 20 = 30.
        // At t=40 both should fire.
        let r2 = mgr.advance(40);
        assert!(r2.contains(&task_delay) && r2.contains(&task_rate));
    }

    #[test]
    fn reschedule_already_scheduled_throws_illegal_state() {
        let mut mgr = TimerManager::new();
        let timer = ObjectId(1);
        let task = ObjectId(10);

        mgr.schedule(timer, task, 50, None, false, 0).unwrap();
        let res = mgr.schedule(timer, task, 50, None, false, 10);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("IllegalStateException"));
    }

    #[test]
    fn cancel_timer_stops_all_tasks() {
        let mut mgr = TimerManager::new();
        let timer = ObjectId(1);
        let task1 = ObjectId(10);
        let task2 = ObjectId(11);

        mgr.schedule(timer, task1, 100, None, false, 0).unwrap();
        mgr.schedule(timer, task2, 100, None, false, 0).unwrap();

        mgr.cancel_timer(timer);
        assert_eq!(mgr.advance(150), vec![]);
    }
}
