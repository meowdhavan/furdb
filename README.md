<div align="center">
  <h1>FurDB</h1>

[![Docker Image CI](https://github.com/meowdhavan/furdb/actions/workflows/docker-image.yml/badge.svg)](https://github.com/meowdhavan/furdb/actions)
[![Minimum rustc 1.88](https://img.shields.io/badge/rustc-1.88+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![oq3_semantics crate](https://img.shields.io/crates/v/furdb.svg)](https://crates.io/crates/furdb)
[![Docker Image Size (latest)](https://img.shields.io/docker/image-size/madhavanraja/furdb/latest)](https://hub.docker.com/r/madhavanraja/furdb)

</div>

A minimal Database Management System that prioritizes storage space usage and fast lookup/query times. **FurDB** lets you specify the specific number of bits occupied by your data.

```
10011100 01010000
┌─┐┌───────┐┌───┐
  ^        ^    ^
  d1       d2   d3
```

## Installation

### Cargo

**FurDB** can be installed using `cargo`.

```sh
cargo install furdb
```

### Compiling from Source

You can clone this repository, build and run the program.

```sh
git clone https://github.com/madhavan-raja/furdb.git
cd ./furdb
cargo build --release
```

`protoc` is **not** required — the build script vendors its own copy to compile
`proto/furdb.proto`.

## Starting the Server

### Docker

You can pull an image and run it in a container.

```sh
docker run --name furdb -d madhavanraja/furdb:latest
```

You can clone this repository, build and run the container using `compose`.

```sh
git clone https://github.com/madhavan-raja/furdb.git
cd ./furdb
docker-compose up --build
```

You can use the image as a service using `compose` in another application.

```yaml
version: "3"
services:
  furdb:
    image: madhavanraja/furdb:latest
    environment:
      WORKDIR: /furdb
      PORT: 5678
    restart: on-failure
```

The server can be reached at `furdb:{PORT}` over gRPC (h2c, no TLS).

### Command Line

If the executable is present in your `PATH`, you can run the server from the command line.

```sh
furdb --workdir "/furdb" serve --port 5678
```

You can use the `help` command to see all the available options.

```sh
furdb help
```

## Usage

**FurDB Server** exposes a gRPC service, `furdb.FurDb`, for creating, reading, and
deleting databases, tables, and entries. The service definition lives in
[`proto/furdb.proto`](proto/furdb.proto).

The examples below use [`grpcurl`](https://github.com/fullstorydev/grpcurl), which
takes the schema from the repository:

```sh
grpcurl -plaintext -proto proto/furdb.proto -d '{ … }' \
  localhost:5678 furdb.FurDb/<Method>
```

### The Response Envelope

Every successful response carries the same envelope, with the operation-specific
payload under `response`:

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {}
}
```

Failures do not use the envelope — they come back as gRPC statuses:

| Failure               | gRPC code          |
| --------------------- | ------------------ |
| Not Found             | `NOT_FOUND`        |
| Bad Request           | `INVALID_ARGUMENT` |
| Conflict              | `ALREADY_EXISTS`   |
| Internal Server Error | `INTERNAL`         |

The `statusCode`/`status` pair is echoed in the `x-furdb-status-code` and
`x-furdb-status` trailers, so it is still available on the error path.

### A Note on Numbers

A column may be up to 128 bits wide, and protobuf has no 128-bit integer type, so
the values stored in one travel as **decimal strings** (`"21"`, not `21`). Column
sizes and indices are bounded and stay numeric.

### Checking Server Info

Gets server information.

**Method**

`furdb.FurDb/GetServerInfo`

**Request**

```json
{}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "workdir": "/furdb"
  }
}
```

### Create Database

Create a database with ID `my_database`.

**Method**

`furdb.FurDb/CreateDatabase`

**Request**

```json
{
  "databaseId": "my_database"
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 201,
  "status": "Created",
  "response": {
    "databaseId": "my_database"
  }
}
```

### Get Database Info

Get info of database with ID `my_database`.

**Method**

`furdb.FurDb/GetDatabase`

**Request**

```json
{
  "databaseId": "my_database"
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "databaseId": "my_database",
    "databaseTables": []
  }
}
```

### Delete Database

Delete database with ID `my_database`.

**Method**

`furdb.FurDb/DeleteDatabase`

**Request**

```json
{
  "databaseId": "my_database"
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK"
}
```

### Create Table

Creates a table with ID `my_table` in the database with ID `my_database`.

**Method**

`furdb.FurDb/CreateTable`

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "tableColumns": [
    {
      "size": 5
    },
    {
      "size": 3
    }
  ]
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 201,
  "status": "Created",
  "response": {
    "databaseId": "my_database",
    "tableId": "my_table",
    "tableColumns": [
      {
        "size": 5
      },
      {
        "size": 3
      }
    ]
  }
}
```

### Get Table Info

Get info of table with ID `my_table` in the database with ID `my_database`.

**Method**

`furdb.FurDb/GetTable`

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table"
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "databaseId": "my_database",
    "tableId": "my_table",
    "tableColumns": [
      {
        "size": 5
      },
      {
        "size": 3
      }
    ]
  }
}
```

### Delete Table

Delete table with ID `my_table` in the database with ID `my_database`.

**Method**

`furdb.FurDb/DeleteTable`

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table"
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK"
}
```

### Insert Entries

Insert entries into table with ID `my_table` in the database with ID `my_database`.

**Method**

`furdb.FurDb/InsertEntries`

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "data": [
    { "data": ["21", "0"] },
    { "data": ["17", "1"] },
    { "data": ["23", "2"] },
    { "data": ["9", "0"] },
    { "data": ["31", "1"] },
    { "data": ["0", "2"] }
  ]
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 201,
  "status": "Created"
}
```

### Get Entries

Get entries from table with ID `my_table` in the database with ID `my_database`.

**Method**

`furdb.FurDb/GetEntries`

#### Get All Entries

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "all": {}
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "resultCount": 6,
    "results": [
      {
        "index": 0,
        "data": ["21", "0"]
      },
      {
        "index": 1,
        "data": ["17", "1"]
      },
      {
        "index": 2,
        "data": ["23", "2"]
      },
      {
        "index": 3,
        "data": ["9", "0"]
      },
      {
        "index": 4,
        "data": ["31", "1"]
      },
      {
        "index": 5,
        "data": ["0", "2"]
      }
    ]
  }
}
```

#### Get Entries By Indices

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "indices": {
    "indices": [1, 3]
  }
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "resultCount": 2,
    "results": [
      {
        "index": 1,
        "data": ["17", "1"]
      },
      {
        "index": 3,
        "data": ["9", "0"]
      }
    ]
  }
}
```

#### Get Entries By Value

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "value": {
    "columnIndex": 0,
    "value": "23"
  }
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "resultCount": 1,
    "results": [
      {
        "index": 2,
        "data": ["23", "2"]
      }
    ]
  }
}
```

### Delete Entries

Delete entries from table with ID `my_table` in the database with ID `my_database`.

**Method**

`furdb.FurDb/DeleteEntries`

#### Delete All Entries

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "all": {}
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK"
}
```

#### Delete Entries By Indices

**Request**

```json
{
  "databaseId": "my_database",
  "tableId": "my_table",
  "indices": {
    "indices": [1]
  }
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK"
}
```
