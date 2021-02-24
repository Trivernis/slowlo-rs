# Slowlo-rs

A project to demonstrate Slowloris HTTP DoS attacks.

DON NOT USE THIS PROGRAM AGAINST SERVERS WITHOUT AUTHORIZATION.


## How does it work?

Slowlorris is an attack on the protocol layer. It establishes a lot of HTTP connections to a webserver
and sends data periodically to keep the connection alive. This causes the servers connection pool
to be fully depleted and it can't accept further connections.

## Usage

```
USAGE:
    slowlo-rs [FLAGS] [OPTIONS] <address>

FLAGS:
        --help       Prints help information
    -h, --http       If the connection should use plain http
    -V, --version    Prints version information

OPTIONS:
    -n, --count <count>    Number of connections [default: 200]
    -p, --port <port>       [default: 443]

ARGS:
    <address>    The target address with port
```

## License

This project is licensed under the MIT-License.