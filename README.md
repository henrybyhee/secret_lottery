# Secret Lottery

# How it works:
1. Admin creates pool. Pool is OPEN.
2. Player delegate funds to pool.
3. Admin locks pool. Delegated coins are staked to a validator node. Pool is LOCKED.
4. After 21 days, Admin releases the fund and closes the pool. A winner will ebe slected randomly.
5. User can claim their fund/reward. Pool is CLOSED.

# Pool Explained:

## States:

1. OPEN:
    - Pool is accepting delegation from players.
    - Validator node is selected by admin
    - Stays open for x days.

2. LOCKED:
    - Admin locks the pool after x days.
    - Delegated funds are sent to validator node for staking.
    - No more delegation from players

3. CLOSED:
    - Funds are released from validator node.
    - Winner is picked randomly.
    - Users can claim their reward.
    - New pool can be created.
