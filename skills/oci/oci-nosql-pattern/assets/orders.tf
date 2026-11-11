# Illustrative NEW-table module; review and pin provider+lockfile before use.
terraform {
  required_version = ">= 1.5.0"
  required_providers {
    oci = { source = "oracle/oci" }
  }
}
variable "region" { type = string }
variable "compartment_id" { type = string }
variable "table_name" {
  type    = string
  default = "orders_v1"
  validation {
    condition     = can(regex("^[A-Za-z][A-Za-z0-9_]{0,63}$", var.table_name))
    error_message = "Use the approved new table name (example caps names at 64 characters)."
  }
}
provider "oci" { region = var.region }
locals {
  ddl = replace(trimspace(file("${path.module}/orders.ddl")), "CREATE TABLE orders_v1", "CREATE TABLE ${var.table_name}")
}
resource "oci_nosql_table" "orders" {
  compartment_id      = var.compartment_id
  name                = var.table_name
  ddl_statement       = local.ddl
  is_auto_reclaimable  = false
  freeform_tags        = { purpose = "orders-example" }
  table_limits {
    capacity_mode      = "PROVISIONED"
    max_read_units     = 50
    max_write_units    = 50
    max_storage_in_gbs = 1
  }
  lifecycle { prevent_destroy = true }
}
resource "oci_nosql_index" "tenant_status_created" {
  name             = "tenant_status_created"
  table_name_or_id = oci_nosql_table.orders.id
  compartment_id   = var.compartment_id
  is_if_not_exists = false
  keys { column_name = "tenantId" }
  keys { column_name = "status" }
  keys { column_name = "createdAt" }
  lifecycle { prevent_destroy = true }
}
output "table_ocid" { value = oci_nosql_table.orders.id }
