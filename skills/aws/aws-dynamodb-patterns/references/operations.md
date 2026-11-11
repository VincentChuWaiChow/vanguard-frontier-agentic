# Amazon DynamoDB operations

Operational coverage for Amazon DynamoDB (control-plane and data-plane APIs; on-demand and provisioned capacity modes; single-Region tables and global tables). Every section uses the
same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[DP-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

## Backup and restore

**Status:** VERIFIED

- Two backup types: on-demand backups (a table snapshot that DynamoDB stores and manages, created by DynamoDB or by AWS Backup) and continuous point-in-time recovery (PITR) backups. Each on-demand backup copies the entire table and there is no limit on how many you can take [DP-E601]. AWS Backup can run backup plans with schedules and retention policies for DynamoDB tables [DP-E605].
- PITR recovery period is configurable from 1 to 35 days [DP-E401]; `LatestRestorableDateTime` is typically about five minutes behind the current time [DP-E402]. Disabling and re-enabling PITR resets the start of the recoverable history [DP-E404].
- Backups and restores consume no provisioned throughput on the table [DP-E602]. On-demand backups are asynchronous and not causally consistent across items: changes within one minute of the request may or may not be included [DP-E416].
- Restore target is always a new table, never in place: PITR restore creates a new table [DP-E410], restore from a backup creates a new table from a backup ARN [DP-E415], and an existing table is not overwritten [DP-E604].
- A restore can change GSIs, LSIs, billing mode, provisioned capacity and encryption settings, and can target a different Region from where the backup resides [DP-E603].
- Not restored automatically (set them again on the restored table): auto scaling policies, IAM policies, CloudWatch metrics and alarms, tags, stream settings, TTL, deletion protection and PITR settings [DP-E411].
- Deleting a table with PITR enabled creates a system backup retained for 35 days [DP-E413]. All backups are automatically encrypted [DP-E607]; an on-demand backup made before a key change keeps its original key and needs it to restore [DP-E347].
- Tags and ABAC are not supported on DynamoDB-native backups; AWS Backup is the documented route for ABAC with backups [DP-E606].

## Disaster recovery and replication

**Status:** VERIFIED

- Single-Region options are PITR and backups. The disaster recovery guide lists PITR with an RPO of seconds and an RTO of minutes, in a single Region only [DP-E608].
- Multi-Region option is global tables, in two consistency modes: multi-Region eventual consistency (MREC) and multi-Region strong consistency (MRSC) [DP-E144] [DP-E146]. MRSC mode cannot be changed after creation [DP-E431].
- Stated availability SLA: 99.999% for global tables versus 99.99% for single-Region tables [DP-E609].
- Stated RPO: MREC equals the replication delay between replicas, usually a few seconds [DP-E610]; MRSC supports an RPO of zero [DP-E611]. MREC replication is asynchronous and a strongly consistent read at a replica can miss a write from another Region [DP-E144]. No RTO figure is stated for global tables on the pages read here.
- Failover mode: the documentation describes shifting application traffic to a healthy replica table, which accepts reads and writes [DP-E612]. Replica placement is your decision: MRSC needs exactly three Regions (three replicas, or two plus a witness) [DP-E429].
- MREC transactions are not replicated atomically as a group [DP-E145]. PITR, tags, deletion protection and resource policies are not synchronized between replicas, so verify them per Region [DP-E426].
- Multi-account global tables are MREC only and have a different settings contract [DP-E436]. The MREC replication stream cannot be disabled [DP-E442].
- Cross-Region recovery from backups is separate from replication: a backup can be restored into a different Region [DP-E603].

## Cost

**Status:** VERIFIED

Pricing: <https://aws.amazon.com/dynamodb/pricing/>. Estimate with the [AWS Pricing Calculator](https://calculator.aws/). No unit prices are recorded here.

- Throughput, on-demand: billed per read request unit and write request unit [DP-E613]. Reads are 4 KB increments and writes 1 KB increments; query capacity rounds the total returned size, and filters or projections do not reduce consumed capacity [DP-E127] [DP-E131].
- Throughput, provisioned: billed on the hourly provisioned read and write capacity, not on consumption [DP-E614]. Reserved capacity is an upfront fee plus a commitment to a minimum provisioned level [DP-E626].
- Storage: billed on raw item bytes plus a per-item overhead that depends on enabled features [DP-E615]. Usage types differ by table class (Standard or Standard-IA) [DP-E616].
- Backup: PITR is billed on table size for as long as PITR is enabled [DP-E617]; reducing the recovery window does not reduce the PITR price [DP-E406]. On-demand backups are billed on size and duration [DP-E618]. Restores are billed on total GB restored, including indexes [DP-E619].
- Replication: global table writes are billed per replica as replicated write units, GSI updates use standard write units [DP-E620]. There is no cross-Region transfer fee for global table replication [DP-E621]. TTL deletes replicate and are charged in the other Regions [DP-E425].
- Streams: DynamoDB Streams reads are billed in read request units [DP-E622]; Lambda trigger reads and global table replication reads are not charged [DP-E623].
- Data transfer: no charge for inbound transfer or same-Region transfer to other AWS services; cross-Region transfer is charged on both sides [DP-E624]. Interface (PrivateLink) endpoints are billed, gateway endpoints are not [DP-E648].
- Adjacent services: Contributor Insights is billed under CloudWatch [DP-E625]; CloudTrail data events and the CloudTrail logs of replication reads are billed by CloudTrail [DP-E660].

## Monitoring

**Status:** PARTIAL

- DynamoDB publishes metrics to Amazon CloudWatch in the `AWS/DynamoDB` namespace [DP-E664]; consumed provisioned throughput is emitted per one-minute period [DP-E663].
- Capacity and throttling metrics: `ConsumedReadCapacityUnits` and `ConsumedWriteCapacityUnits` (add `GlobalSecondaryIndexName` for an index) [DP-E501]; `ReadThrottleEvents` and `WriteThrottleEvents` [DP-E502]; `ThrottledRequests` and `SystemErrors`, which need both `TableName` and `Operation` [DP-E503]; `UserErrors` is a Regional aggregate without a table dimension [DP-E505].
- Cause-specific throttle metrics include `ReadKeyRangeThroughputThrottleEvents` for partition limits [DP-E633].
- Global tables: MREC publishes `ReplicationLatency` with `TableName` and `ReceivingRegion` dimensions [DP-E509]; MRSC does not publish it [DP-E511].
- Usage metrics integrate with Service Quotas, and CloudWatch alarms can alert when usage approaches a threshold [DP-E665].
- PARTIAL because: a provider-recommended alarm set was not found on the pages read (searched the metrics and dimensions page, the throttling pages and the monitoring overview, which is JavaScript-rendered and returned no text). Alarm thresholds are therefore your own choice.

## Performance

**Status:** VERIFIED

- Each partition is designed for a maximum of 3,000 read units and 1,000 write units per second [DP-E627]. Adaptive capacity applies to both on-demand and provisioned modes [DP-E628].
- Throttling has four documented causes: partition (key range) limits in both modes, provisioned throughput exceeded, account-level quotas in on-demand mode, and a configured on-demand maximum throughput [DP-E629]. Each maps to a cause-specific metric [DP-E633].
- The throttle exception carries `ThrottlingReason` fields naming the resource, operation and limit [DP-E632]. The AWS SDKs automatically retry `ProvisionedThroughputExceededException` [DP-E635].
- On-demand scaling: new tables sustain up to 4,000 writes and 12,000 reads per second [DP-E636]; tables accommodate up to double the previous peak instantly [DP-E630], and throttling can occur if you exceed double the previous peak within 30 minutes [DP-E631]. An optional maximum throughput per table or index throttles requests above it [DP-E634].
- Index pressure: an overloaded GSI can throttle base-table writes [DP-E107]; on-demand GSIs still have limits and hot keys [DP-E116].
- An LSI item collection lives in one partition and is limited to 10 GB per partition key value [DP-E108] [DP-E122].

## IAM

**Status:** VERIFIED

- Identity-based policies use `dynamodb:*` actions. Control-plane actions (`CreateTable`, `UpdateTable`, `DeleteTable`, `DescribeTable`, TTL and continuous-backup actions) apply to the table resource [DP-E307]; `ListTables`, `ListStreams`, `DescribeLimits` and `DescribeEndpoints` have no resource-level scope [DP-E309].
- Resource ARNs: tables `table/<name>`, indexes `table/<name>/index/<index>`, streams `table/<name>/stream/<label>` [DP-E301].
- Built-in managed policies: `AmazonDynamoDBFullAccess` [DP-E637] and `AmazonDynamoDBReadOnlyAccess` [DP-E638]. They are broad, so prefer scoped custom policies for production.
- Least privilege levers: restrict by table and index ARN [DP-E301]; `dynamodb:LeadingKeys` for item-level scoping on supported actions (not `Scan`) [DP-E315]; tag conditions with ABAC [DP-E321].
- Resource-based policies are supported for tables, indexes and streams [DP-E639]; support differs by API and cross-account use [DP-E325]. IAM Access Analyzer reports cross-account access in them [DP-E640], and Block Public Access is enforced automatically with no setting [DP-E641].
- Global tables use a service-linked role; an explicit deny of its replication action breaks adding or deleting replicas [DP-E331]. Replica creation needs specific permissions on source and destination [DP-E421].

## Tagging

**Status:** PARTIAL

- Tables can be tagged [DP-E642], tags feed AWS cost reports [DP-E643], and indexes inherit the tags of their table [DP-E317]. Tag operations (`TagResource`, `UntagResource`, `ListTagsOfResource`) apply to a table or stream [DP-E310].
- Tag-based access control (ABAC) is documented for tables and, separately, for Streams, with enablement scoped per account and Region [DP-E321]. Behaviour of tag conditions when ABAC is disabled differs [DP-E323].
- Not covered: DynamoDB-native backups cannot be tagged and do not support ABAC [DP-E606]. Tags are not restored with a table [DP-E411] and are not synchronized between global table replicas [DP-E426].
- PARTIAL because two official pages disagree on Streams: the Tagging page says Streams usage cannot be tagged, while the Streams tagging page documents adding tags to existing streams [DP-E644] [DP-E318]. Test before relying on stream tags.

## Encryption

**Status:** VERIFIED

- At rest: server-side encryption is always on and cannot be disabled; LSIs and GSIs use the base table key [DP-E334] [DP-E120].
- Key choices: AWS owned key (default, no charge), AWS managed key `alias/aws/dynamodb`, or customer managed key; the latter two incur AWS KMS charges [DP-E335]. For the AWS managed key you can view the policy and audit use in CloudTrail [DP-E336].
- Customer managed key restrictions: symmetric keys only [DP-E337]; the key policy must grant DynamoDB access, at minimum including `kms:CreateGrant` [DP-E346]; disabling or deleting the key, or removing grants, can block table access [DP-E349]. Global tables add a customer managed key version requirement [DP-E350].
- Changing keys with `update-table` switches the table key [DP-E343]; older on-demand backups keep their original key and need it to restore [DP-E347]. Table keys are cached, so KMS is not called for every item access [DP-E338].
- DAX cluster encryption is separate and does not support a customer managed key [DP-E351]. All backups are automatically encrypted [DP-E607].
- In transit: clients must support TLS 1.2 [DP-E645].

## Network isolation

**Status:** PARTIAL

- Two VPC endpoint types reach DynamoDB: a gateway endpoint (a route-table target over the AWS network) and an interface endpoint using AWS PrivateLink [DP-E646]. Both can coexist in one VPC [DP-E648].
- Interface endpoints give VPC applications private IP access without public IP addresses [DP-E647]. They are billed; gateway endpoints are not [DP-E648]. PrivateLink is also supported for DynamoDB Streams endpoints [DP-E652].
- Scope access with a VPC endpoint policy (principal, actions, resources) [DP-E651]. For requests via a VPC endpoint, both the IAM policy and the endpoint policy must authorize `DescribeEndpoints` [DP-E653].
- PrivateLink limits: 50,000 requests per second per endpoint [DP-E650]; do not override DynamoDB DNS names with private hosted zones, point clients at the endpoint URL instead [DP-E649].
- PARTIAL because: the documentation read here describes private paths and policy scoping but no setting that turns off the public DynamoDB endpoint for an account or table. Block Public Access applies to resource-based policies, not to the network path [DP-E641]. Two pages also differ on on-premises reach of "VPC endpoints" (Site-to-Site VPN and Direct Connect) without saying which endpoint type the older note means [DP-E654].

## Audit logging

**Status:** VERIFIED

- AWS CloudTrail records DynamoDB API calls from the console, SDKs, PartiQL and the classic API [DP-E655]. Without a trail, recent events stay in CloudTrail Event history [DP-E661].
- Control-plane events (table, backup, global table, tag, TTL, DAX and Streams describe or list actions) are logged by default [DP-E656]. Backup and restore actions are recorded [DP-E662].
- Data-plane (item-level) auditing is possible but opt-in: you must enable logging of data plane API activity as CloudTrail data events [DP-E657].
- Data events can be filtered by resource type (table or stream) to control what is logged and billed [DP-E658]. TTL deletions are not logged [DP-E659].
- Logging `GetRecords` data events also captures internal global tables replication calls and is billed by CloudTrail, so filter them out [DP-E660].
