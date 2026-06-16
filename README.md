<div align="center">
  <h1>FurDB</h1>

[![Docker Image CI](https://github.com/meowdhavan/furdb/actions/workflows/docker-image.yml/badge.svg)](https://github.com/meowdhavan/furdb/actions)
[![Minimum rustc 1.70](https://img.shields.io/badge/rustc-1.70+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
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

The server can be accessed at `http://furdb:{PORT}`.

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

**FurDB Server** provides REST API endpoints for creating, reading, and deleting databases, tables, and entries.

### Checking Server Info

Gets server information.

**Endpoint**

`GET` `/`

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": {
    "message": "Server is running",
    "config": {
      "workdir": "/furdb"
    }
  }
}
```

### Create Database

Create a database with ID `my_database`.

**Endpoint**

`POST` `/my_database`

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

**Endpoint**

`GET` `/my_database`

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

**Endpoint**

`DELETE` `/my_database`

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": null
}
```

### Create Table

Creates a table with ID `my_table` in the database with ID `my_database`.

**Endpoint**

`POST` `/my_database/my_table`

**Request**

```json
{
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

**Endpoint**

`GET` `/my_database/my_table`

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

**Endpoint**

`DELETE` `/my_database/my_table`

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": null
}
```

### Insert Entries

Insert entries into table with ID `my_table` in the database with ID `my_database`.

**Endpoint**

`POST` `/my_database_/my_table/data`

**Request**

```json
{
  "data": [
    [21, 0],
    [17, 1],
    [23, 2],
    [9, 0],
    [31, 1],
    [0, 2]
  ]
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 201,
  "status": "Created",
  "response": null
}
```

### Get Entries

Get entries from table with ID `my_table` in the database with ID `my_database`.

**Endpoint**

`GET` `/my_database_/my_table/data`

#### Get All Entries

**Request**

```json
{
  "entries": "all"
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
        "data": [21, 0]
      },
      {
        "index": 1,
        "data": [17, 1]
      },
      {
        "index": 2,
        "data": [23, 2]
      },
      {
        "index": 3,
        "data": [9, 0]
      },
      {
        "index": 4,
        "data": [31, 1]
      },
      {
        "index": 5,
        "data": [0, 2]
      }
    ]
  }
}
```

#### Get Entries By Indices

**Request**

```json
{
  "entries": {
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
        "data": [17, 1]
      },
      {
        "index": 3,
        "data": [9, 0]
      }
    ]
  }
}
```

#### Get Entries By Value

**Request**

```json
{
  "entries": {
    "value": {
      "columnIndex": 0,
      "value": 23
    }
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
        "data": [23, 2]
      }
    ]
  }
}
```

### Delete Entries

Delete entries from table with ID `my_table` in the database with ID `my_database`.

**Endpoint**

`DELETE` `/:database_id/:table_id/data`

#### Delete All Entries

**Request**

```json
{
  "entries": "all"
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": null
}
```

#### Delete Entries By Indices

**Request**

```json
{
  "entries": {
    "indices": [1]
  }
}
```

**Response**

```json
{
  "result": "success",
  "statusCode": 200,
  "status": "OK",
  "response": null
}
```
