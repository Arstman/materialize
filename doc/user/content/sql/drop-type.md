---
title: "DROP TYPE"
description: "`DROP TYPE` removes a user-defined data type."
menu:
  main:
    parent: commands
---

`DROP TYPE` removes a [custom data type](../create-type). You cannot use it on default data types.

## Syntax

{{< diagram "drop-type.svg" >}}

Field | Use
------|-----
**IF EXISTS**  | Do not return an error if the named type doesn't exist.
_data_type_name_ | The name of the type to remove.
**CASCADE** | Remove the type and its dependent objects, such as tables or other types.
**RESTRICT** |  Don't remove the type if any objects depend on it. _(Default.)_

## Examples

### Remove a type with no dependent objects
```mzsql
CREATE TYPE int4_map AS MAP (KEY TYPE = text, VALUE TYPE = int4);

SHOW TYPES;
```
```
    name
--------------
  int4_map
(1 row)
```

```mzsql
DROP TYPE int4_map;

SHOW TYPES;
```
```
  name
--------------
(0 rows)
```

### Remove a type with dependent objects

By default, `DROP TYPE` will not remove a type with dependent objects. The **CASCADE** switch will remove both the specified type and *all its dependent objects*.

In the example below, the **CASCADE** switch removes `int4_list`, `int4_list_list` (which depends on `int4_list`), and the table *t*, which has a column of data type `int4_list`.

```mzsql
CREATE TYPE int4_list AS LIST (ELEMENT TYPE = int4);

CREATE TYPE int4_list_list AS LIST (ELEMENT TYPE = int4_list);

CREATE TABLE t (a int4_list);

SHOW TYPES;
```
```
      name
----------------
 int4_list
 int4_list_list
(2 rows)
```

```mzsql
DROP TYPE int4_list CASCADE;

SHOW TYPES;

SELECT * FROM t;
```
```
 name
------
(0 rows)
ERROR:  unknown catalog item 't'
```

### Remove a type only if it has no dependent objects

You can use either of the following commands:

- ```mzsql
  DROP TYPE int4_list;
  ```
- ```mzsql
  DROP TYPE int4_list RESTRICT;
  ```

### Do not issue an error if attempting to remove a nonexistent type

```mzsql
DROP TYPE IF EXISTS int4_list;
```

## Privileges

The privileges required to execute this statement are:

{{< include-md file="shared-content/sql-command-privileges/drop-type.md" >}}

## Related pages

* [`CREATE TYPE`](../create-type)
* [`SHOW TYPES`](../show-types)
* [`DROP OWNED`](../drop-owned)
