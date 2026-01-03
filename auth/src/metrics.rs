//! Metrics for authentication operations.
//!
//! These metrics are recorded when authentication events occur.
//! The metrics exporter must be initialized by the consuming application.
//!
//! Uses `ethpayserver_` prefix for consistency with server metrics.

use metrics::counter;

/// Record a successful user registration.
pub fn record_user_registration() {
    counter!("ethpayserver_user_registrations_total").increment(1);
}

/// Record a successful user login.
pub fn record_user_login() {
    counter!("ethpayserver_user_logins_total").increment(1);
}

/// Record a user logout.
pub fn record_user_logout() {
    counter!("ethpayserver_user_logouts_total").increment(1);
}

/// Record a failed login attempt.
pub fn record_login_failure() {
    counter!("ethpayserver_user_login_failures_total").increment(1);
}
