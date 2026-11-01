use crate::heap::object::ObjectId;
use crate::vm::frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    Ready,
    Running,
    Sleeping {
        wake_time_ms: u64,
    },
    Waiting {
        monitor: Option<ObjectId>,
        timeout_wake_ms: Option<u64>,
        saved_recursion: usize,
    },
    Blocked {
        monitor: ObjectId,
        pending_recursion: usize,
    },
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExceptionEvent {
    pub exception_class: String,
    pub message: Option<String>,
    pub thread_name: String,
    pub class_name: String,
    pub method_name: String,
    pub descriptor: String,
    pub pc: usize,
    pub caught: bool,
    pub stack_trace: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GreenThread {
    pub id: ThreadId,
    pub name: String,
    pub java_thread_obj: Option<ObjectId>,
    pub call_stack: Vec<Frame>,
    pub state: ThreadState,
    pub priority: i32,
    pub instructions_executed: u64,
    pub exceptions_thrown: u64,
    pub exceptions_caught: u64,
    pub exceptions_uncaught: u64,
    pub auto_healing_count: u32,
    pub last_auto_healing_signature: Option<String>,
    pub last_exception_event: Option<ExceptionEvent>,
    pub auto_healing_events: Vec<ExceptionEvent>,
    pub pending_monitor_releases: Vec<ObjectId>,
    pub wait_start_instructions: u64,
    pub yield_count_in_frame: usize,
    pub consecutive_yields: usize,
}

impl GreenThread {
    pub fn new(
        id: ThreadId,
        name: String,
        initial_frame: Frame,
        java_thread_obj: Option<ObjectId>,
    ) -> Self {
        Self {
            id,
            name,
            java_thread_obj,
            call_stack: vec![initial_frame],
            state: ThreadState::Ready,
            priority: 5, // NORM_PRIORITY in Java
            instructions_executed: 0,
            exceptions_thrown: 0,
            exceptions_caught: 0,
            exceptions_uncaught: 0,
            auto_healing_count: 0,
            last_auto_healing_signature: None,
            last_exception_event: None,
            auto_healing_events: Vec::new(),
            pending_monitor_releases: Vec::new(),
            wait_start_instructions: 0,
            yield_count_in_frame: 0,
            consecutive_yields: 0,
        }
    }

    pub fn on_yield(&mut self) {
        self.yield_count_in_frame += 1;
        self.consecutive_yields += 1;
    }

    pub fn is_alive(&self) -> bool {
        self.state != ThreadState::Terminated && !self.call_stack.is_empty()
    }

    pub fn current_frame(&self) -> Option<&Frame> {
        self.call_stack.last()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut Frame> {
        self.call_stack.last_mut()
    }
}
