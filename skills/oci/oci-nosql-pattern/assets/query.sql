DECLARE $tenant STRING; $status STRING; SELECT o.tenantId, o.orderId, o.status, o.createdAt FROM orders_v1 o WHERE o.tenantId = $tenant AND o.status = $status ORDER BY o.createdAt
