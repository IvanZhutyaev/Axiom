//! Online ML: linear regression, streaming GBDT stub, MLP, anomaly detection.

pub mod anomaly;
pub mod boosting;
pub mod linear;
pub mod mlp;

pub use anomaly::AnomalyDetector;
pub use boosting::StreamingGbdt;
pub use linear::LinearModel;
pub use mlp::MlpModel;
