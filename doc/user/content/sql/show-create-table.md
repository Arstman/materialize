---
title: "SHOW CREATE TABLE"
description: "`SHOW CREATE TABLE` returns the SQL used to create the table."
menu:
  main:
    parent: commands
---

`SHOW CREATE TABLE` returns the SQL used to create the table.

## Syntax

```sql
SHOW [REDACTED] CREATE TABLE <table_name>
```

{{< yaml-table data="show_create_redacted_option" >}}

For available table names, see [`SHOW TABLES`](/sql/show-tables).

## Examples

```mzsql
CREATE TABLE t (a int, b text NOT NULL);
```

```mzsql
SHOW CREATE TABLE t;
```
```nofmt
         name         |                                             create_sql
----------------------+-----------------------------------------------------------------------------------------------------
 materialize.public.t | CREATE TABLE "materialize"."public"."t" ("a" "pg_catalog"."int4", "b" "pg_catalog"."text" NOT NULL)
```

## Privileges

The privileges required to execute this statement are:

{{< include-md file="shared-content/sql-command-privileges/show-create-table.md"
>}}

## Related pages

- [`SHOW TABLES`](../show-tables)
- [`CREATE TABLE`](../create-table)
