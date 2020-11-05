use crate::msg::{HandleMsg, InitMsg, OwnerResponse, PoolResponse, QueryMsg};
use crate::state::{config, config_read, pool_read, pool_storage, Pool, PoolStatus, State, DAYS};
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HandleResult, InitResponse,
    Querier, StdError, StdResult, Storage,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        count: 0,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };
    config(&mut deps.storage).save(&state)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::CrtePool {} => admin_create_pool(deps, env),
        HandleMsg::LockPool {} => admin_lock_pool(deps, env),
        HandleMsg::ClsePool {} => admin_close_pool(deps, env),
    }
}

fn assert_sender_is_admin(sender: CanonicalAddr, owner: CanonicalAddr) -> StdResult<()> {
    if owner != sender {
        return Err(StdError::unauthorized());
    }
    Ok(())
}

// Create a new pool.
pub fn admin_create_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // Ensure that only contract owner can create the pool
    let state = config_read(&deps.storage).load()?;
    let sender_addr = deps.api.canonical_address(&env.message.sender)?;
    assert_sender_is_admin(sender_addr, state.owner)?;
    // Can only create a new pool if:
    // 1. No pool is available
    // 2. Previous Pool is CLOSED.
    let res = pool_read(&deps.storage).load();
    let can_create = res.as_ref().map_or(true, |x| x.is_closed());
    if !can_create {
        return Err(StdError::generic_err("Cannot create"));
    }
    // Create the pool and persist it.
    let new_pool = Pool::new(env.block.time);
    pool_storage(&mut deps.storage).save(&new_pool)?;
    Ok(HandleResponse::default())
}

// Lock the pool.
// TODO:
// - Send all funds to validator.
// Edge Case:
// - What happens if Pool has no delegators?
pub fn admin_lock_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // Ensure that only contract owner can create the pool
    let state = config_read(&deps.storage).load()?;
    let sender_addr = deps.api.canonical_address(&env.message.sender)?;
    assert_sender_is_admin(sender_addr, state.owner)?;
    // Only OPEN pool can be locked.
    let mut pool = pool_storage(&mut deps.storage).load()?;
    if !pool.is_open() {
        return Err(StdError::generic_err(
            "Pool must be in OPEN status to be locked.",
        ));
    }
    // Ensure that pool is open for 1 day before locking.
    pool.assert_status_has_expired(env.block.time)?;
    pool.lock(env.block.time);
    pool_storage(&mut deps.storage).save(&pool)?;
    // TODO: Send all funds to validator node.
    Ok(HandleResponse::default())
}

pub fn admin_close_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // Ensure that only contract owner can create the pool
    let state = config_read(&deps.storage).load()?;
    let sender_addr = deps.api.canonical_address(&env.message.sender)?;
    assert_sender_is_admin(sender_addr, state.owner)?;
    // Only LOCKED pool can be closed.
    let mut pool = pool_storage(&mut deps.storage).load()?;
    if !pool.is_locked() {
        return Err(StdError::generic_err("Pool is not LOCKED."));
    }
    // Pool must remain locked for 2 days before closing.
    pool.assert_status_has_expired(env.block.time)?;
    pool.close(env.block.time);
    pool_storage(&mut deps.storage).save(&pool)?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetCurrentPool {} => to_binary(&query_pool(deps)?),
    }
}

// Get owner info
fn query_owner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<OwnerResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(OwnerResponse {
        owner: deps.api.human_address(&state.owner)?,
    })
}

// Get Pool Info
fn query_pool<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<PoolResponse> {
    let pool = pool_read(&deps.storage).load().ok();
    Ok(PoolResponse { pool })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::HumanAddr;
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetOwner {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(HumanAddr::from("creator"), value.owner);
    }

    #[test]
    fn test_create_pool_admin() {
        let mut deps = mock_dependencies(20, &coins(2, "earth"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "earth"));
        init(&mut deps, env, msg).unwrap();

        let mut env = mock_env("creator", &coins(2, "earth"));
        env.block.time = 1000;
        handle(&mut deps, env, HandleMsg::CrtePool {}).unwrap();

        // Get the pool result
        let res = query(&deps, QueryMsg::GetCurrentPool {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        assert_eq!(value.pool, Some(Pool::new(1000)));
    }

    #[test]
    fn test_create_pool_errors() {
        let mut deps = mock_dependencies(20, &coins(2, "earth"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "earth"));
        init(&mut deps, env, msg).unwrap();

        // Only admin can create pool
        let env = mock_env("voter", &coins(2, "earth"));
        let res = handle(&mut deps, env, HandleMsg::CrtePool {});

        assert_eq!(res.is_err(), true);
        assert_eq!(res.unwrap_err(), StdError::unauthorized());
    }

    #[test]
    fn test_lock_pool() {
        let mut deps = mock_dependencies(20, &coins(2, "scrt"));

        // Initialize the contract
        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "scrt"));
        init(&mut deps, env, msg).unwrap();

        // Create the pool
        let mut env = mock_env("creator", &coins(2, "scrt"));
        env.block.time = 1000;
        env.block.height = 1000;
        handle(&mut deps, env, HandleMsg::CrtePool {}).unwrap();

        // Lock the pool.
        let mut env = mock_env("creator", &coins(2, "scrt"));
        env.block.time = DAYS * 21 + 1001;
        env.block.height = DAYS * 21 + 1001;
        handle(&mut deps, env, HandleMsg::LockPool {}).unwrap();

        let res = query(&deps, QueryMsg::GetCurrentPool {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        assert_eq!(value.pool.unwrap().is_locked(), true);
    }
}
