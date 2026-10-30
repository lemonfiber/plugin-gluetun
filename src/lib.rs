//! lemonfiber's adapter for Gluetun: the `network.egress-guard` contract, spoken in front
//! of one Gluetun through its control server, held to the key lemonfiber hands this adapter
//! by the role file Gluetun reads when it starts.

pub mod adapter;
pub mod gluetun;
