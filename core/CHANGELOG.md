# 0.2.1 [2023-03-02]
- Advance poll to Connection definition for type bounded ConnectionHandler.
- Initialize main program.
- Define lifetime bounds for Polling ordering.
- Initialize grpc connection example.
- Implement connection handler traits for GrpcMiddleware types and events.
- Add scatter-gather-grpc example with protobuffer schema.

# 0.2.0 [2023-02-21]

- Add Executor and Handler traits.
- Add Connection type with ConnectionId and NodeConfig.
- Add PoolConnection bounded to Handler trait.
- Add PendingConnection wrapper bounded to Handler trait.
- Add EstablishedConnection wrapper bounded to Handler trait.
- Add PoolEvent bounded to Handler trait.
- Implement fn notify_event() for PoolEvent.
- Add Pool bounded to Handler and Executor traits.
- Add ConnectionCounters and ConnectionLimits to Pool.