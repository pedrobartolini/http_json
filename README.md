# direct_http

An simple and low-level rust REST API library.


## WARNING!
Do not use this. It's complete garbage, I was just messing with the language.<br>
I will come back someday to make it better.

## Response

`Response` is a struct that represents the HTTP response.<br/>
It contains a `status enum` that represents the status code, an optional `message` of type `String`, and optional `data` of type `Value`.

```rust
pub struct Response {
  status: Status,
  message: Option<String>,
  data: Option<Value>,
}
```

You should always construct the Response using the status you want to return.<br/>
Passing an message or data is optional.

#### response

```rust
let response = Response::status(Status::Ok)

```

#### response with data

```rust
let response = Response::status(Status::Ok).data(ENCODE!(user))

```

#### response with message

```rust
let response = Response::status(Status::Ok).message("Hello world!")

```

#### response with data and message

```rust
let response = Response::status(Status::Ok).message("Hello world!").data(ENCODE!(user))

```
