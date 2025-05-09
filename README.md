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
./target/release/krb5proxy --listen 127.0.0.1:8080 --upstream http://upstream-proxy:3128
```

Typical usage:
- Configure your client (e.g., browser, curl) to use `http://127.0.0.1:8080` as its proxy.
- krb5proxy will forward requests to `upstream-proxy:3128` using Kerberos authentication.

## Help

Make sure your Kerberos environment is properly configured. You can validate your ticket using:

```bash
klist
```

If you encounter authentication issues, try renewing or initializing your ticket:

```bash
kinit your-principal@YOUR.REALM
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

