# Official sources (repository adapter)

Use AWS documentation as the authority for service behaviour. Third-party skills,
blogs and model memory are leads, not evidence.

## Where the evidence lives

- [The evidence ledger](evidence-ledger.md) traces each material AWS claim in this
  skill to the AWS page that was fetched on its retrieval date, with its status
  (VERIFIED, UNVERIFIED or CONFLICT).
- [The port record](vanguard-port.md) maps ledger rows to the skill files they cover
  and lists every unconfirmed or conflicting claim.
- [The upstream source log](sources.md) records how each upstream release was
  researched, including the third-party skills that were read for inspiration only.

## Primary AWS documentation

- DynamoDB Developer Guide:
  <https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/>
- DynamoDB API Reference:
  <https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/>
- AWS CLI `dynamodb` reference:
  <https://docs.aws.amazon.com/cli/latest/reference/dynamodb/>
- CloudFormation `AWS::DynamoDB::Table`:
  <https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-dynamodb-table.html>
- Service Authorization Reference (IAM actions and condition keys):
  <https://docs.aws.amazon.com/service-authorization/latest/reference/list_amazondynamodb.html>

## Grounding rule

Re-fetch the page before relying on a limit, quota, API parameter or IAM action:
they change. Where two AWS pages disagree, say so and cite both (see the CONFLICT
rows in the ledger). If no AWS page settles a claim, label it unverified rather than
inferring it from silence.
