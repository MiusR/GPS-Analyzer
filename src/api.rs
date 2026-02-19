pub mod router;
pub mod controller;
pub mod cors;
pub mod state;
pub mod model;
mod repository;
pub mod service; // TODO move init to controller and make this private, also do that for controller, only router should be visible from main
mod middleware;
mod util;