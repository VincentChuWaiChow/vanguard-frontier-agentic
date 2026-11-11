CREATE TABLE orders_v1 (
  tenantId STRING,
  orderId STRING,
  status STRING,
  createdAt TIMESTAMP(3),
  amount NUMBER,
  document JSON,
  PRIMARY KEY(SHARD(tenantId), orderId)
) USING TTL 30 DAYS
