use cosmwasm_std::Uint128;
use cosmwasm_std::{CanonicalAddr, StdError, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub static CONFIG_KEY: &[u8] = b"config";
pub static POOL_KEY: &[u8] = b"pool";
pub const DAYS: u64 = 60 * 60 * 24;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: CanonicalAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PoolStatus {
    // Pool is accepting players.
    OPEN,
    // Pool is locked and acrueing interest.
    LOCKED,
    // Pool is closed and rewards are available.
    CLOSED,
}

// TODO:
//   - Add validator node
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pool {
    pub delegated_amt: Uint128,
    pub delegators: Vec<CanonicalAddr>,
    pub status: PoolStatus,
    pub status_updated_at: u64,
}

impl Pool {
    pub fn new(time: u64) -> Self {
        Pool {
            delegated_amt: Uint128(0),
            delegators: vec![],
            status: PoolStatus::OPEN,
            status_updated_at: time,
        }
    }
    pub fn is_open(&self) -> bool {
        self.status == PoolStatus::OPEN
    }
    pub fn is_locked(&self) -> bool {
        self.status == PoolStatus::LOCKED
    }
    pub fn is_closed(&self) -> bool {
        self.status == PoolStatus::CLOSED
    }
    pub fn lock(&mut self, time: u64) {
        self.status = PoolStatus::LOCKED;
        self.status_updated_at = time;
    }
    pub fn close(&mut self, time: u64) {
        self.status = PoolStatus::CLOSED;
        self.status_updated_at = time;
    }
    pub fn assert_ready_for_status_change(&self, curr_time: u64) -> StdResult<()> {
        match self.status {
            PoolStatus::OPEN => {
                if self.status_updated_at + 1 * DAYS > curr_time {
                    return Err(StdError::generic_err(format!(
                        "Pool has to be OPEN for {} day",
                        1
                    )));
                }
            }
            PoolStatus::LOCKED => {
                if self.status_updated_at + 21 * DAYS > curr_time {
                    return Err(StdError::generic_err(format!(
                        "Pool has to be LOCKED for {} day",
                        21
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn pool_storage<S: Storage>(storage: &mut S) -> Singleton<S, Pool> {
    singleton(storage, POOL_KEY)
}

pub fn pool_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Pool> {
    singleton_read(storage, POOL_KEY)
}
