//! Compliance and approval workflow for AI Real Estate Team
//!
//! Risk assessment, approval queue management, and audit logging.

pub mod approval_engine;
pub mod audit_logger;

pub use approval_engine::{ApprovalEngine, ApprovalQueue};
pub use audit_logger::{AuditLogger, AuditRecord};
