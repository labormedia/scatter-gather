# 0.2.0 [2023-02-21]

- Add Executor and Handler traits.
- Add Connection type with ConnectionId and ServerConfig.
- Add PoolConnection bounded to Handler trait.
- Add PendingConnection wrapper bounded to Handler trait.
- Add EstablishedConnection wrapper bounded to Handler trait.
- Add PoolEvent bounded to Handler trait.
- Implement fn notify_event() for PoolEvent.
- Add Pool bounded to Handler and Executor traits.
- Add ConnectionCounters and ConnectionLimits to Pool.