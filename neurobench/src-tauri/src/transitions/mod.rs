//! Transition Engine Module
//!
//! Comprehensive transition/edge management for FSM design:
//! - Typed transitions with guards, actions, events
//! - Connection validation (port compatibility, cycles)
//! - Priority-based ordering
//! - Integration with Node Engine

pub mod transition_types;
pub mod transition_engine;
pub mod validation;

// Re-export main types
pub use transition_types::{
    Transition,
    TransitionEvent,
    TransitionUpdate,
    TransitionInfo,
    ConnectionPoint,
};

pub use transition_engine::{
    TransitionEngine,
    TransitionError,
};

pub use validation::{
    ConnectionValidator,
    ValidationResult,
    ValidationError,
    PortCompatibility,
};
