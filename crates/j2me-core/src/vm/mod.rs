pub mod class_loader;
pub mod frame;
pub mod instruction;
pub mod interpreter;

pub use class_loader::{parse_param_count, ClassLoader, PredecodedMethod};
pub use frame::Frame;
pub use instruction::{Instruction, Predecoder};
pub use interpreter::{Interpreter, StepResult};
