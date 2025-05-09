# krb5proxy

Kerberos-aware HTTP proxy for environments requiring Proxy-Authorization with Negotiate (Kerberos) scheme.

## Description

krb5proxy is a minimal HTTP forward proxy that transparently injects Kerberos (Negotiate) authentication
to upstream proxies or services requiring `Proxy-Authorization: Negotiate <token>`.
It is designed for environments where clients are unable or unwilling to handle Kerberos themselves.

The proxy acquires and injects Kerberos tokens using system keytab or the current user session
and relays HTTP requests to the next hop, supporting both regular HTTP requests and HTTP CONNECT tunneling.

## Getting Started

### Dependencies

* Linux (tested on Rocky Linux 9, Ubuntu 24.04)
* krb5-libs and krb5-workstation installed (MIT Kerberos)
* Valid Kerberos configuration in `/etc/krb5.conf`
* Valid Kerberos ticket or system keytab for the proxy service

### Installing

Clone this repository and build using Cargo:

```bash
git clone https://github.com/veldrane/krb5proxy.git
cd krb5proxy
cargo build --release
```

The resulting binary will be in `target/release/krb5proxy`.


### Executing program

Prepare your environment with a valid Kerberos ticket (e.g., using `kinit`) or ensure the keytab is accessible.

Run the proxy:

```bash
./target/release/krb5proxy --listen http://127.0.0.1:8080 --proxy http://10.0.0.1:3128 --proxy-hostname proxy.foo.com
```

krb5proxy doesn't resolve ip yet but needs to have a properly configured parrent proxy hostname for getting the right TGS ticket 

Typical usage:
- Configure http_proxy and https_proxy environment variables to to point `http://127.0.0.1:8080`
- krb5proxy will forward requests to `10.0.0.1:3128` using Kerberos authentication.

## Help

Make sure your Kerberos environment is properly configured. You can validate your ticket using:

```bash
klist
```

If you encounter authentication issues, try renewing or initializing your ticket:

```bash
kinit your-principal@YOUR.REALM
```

If you have a right permissions krb5proxy asks for TGS ticket for authentization on the parrent proxy.

For krb5proxy parameters:

```
$ target/release/krb5proxy --help
krb5proxy 0.1.0
A tiny proxy to forward requests with Kerberos authentication to parrent proxy

USAGE:
    krb5proxy [OPTIONS] --proxy <PROXY> --proxy-hostname <PROXY_HOSTNAME>

OPTIONS:
    -h, --help
            Print help information

    -l, --listen <LISTEN>
            Listen address in the format http://<ip>:<port> - default is http://127.0.0.1:8080
            [default: http://127.0.0.1:8080]

    -p, --proxy <PROXY>
            Parrent proxy string in the format <ip>:<port>

        --proxy-hostname <PROXY_HOSTNAME>
            Parent proxy hostname

    -V, --version
            Print version information
```

## Authors

[@veldrane](https://github.com/veldrane)

## Version History

* 0.1.0
    * Initial public release

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Inspired by:
* https://sourceforge.net/projects/cntlm/

