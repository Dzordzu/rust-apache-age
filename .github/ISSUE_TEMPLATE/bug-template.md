---
name: Bug template
about: Report a bug
title: "[Bug] Name of the bug"
labels: bug, waiting-for-reply
assignees: Dzordzu

---

You can get image details using `docker image ls`

# Example issue template



 ## System Information
* OS: linux(result of the `uname -r`) / Windows 11 / Mac (version)
* Image ID: `44e9173a0787`
* Image: [apache/age:PG11_latest](https://hub.docker.com/layers/apache/age/PG11_latest/images/sha256-d3e27eae0e1bf3e0eb12ef7a79352e22990b60b09a16bd76ef79501153bd3b41?context=explore)
* Docker version: `Docker version 24.0.5, build ced0996600`

## Issue

Cannot specify database name

## How to reproduce an issue

Run `docker run apache/age…` 

Example code that panics

```rust
pub fn function_that_panics() {
   // …
}
```

## Expected behavior

Example code doesn't panic
