# krb5proxy

<p align="left">
  <img src="./images/krb5proxy-transparent.png" alt="krb5proxy logo" width="180"/>
</p>

![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)
![Build](https://github.com/veldrane/krb5proxy/actions/workflows/build.yml/badge.svg)

Kerberos-aware HTTP proxy for environments requiring Proxy-Authorization with Negotiate (Kerberos) scheme.

## Description

krb5proxy is a minimal HTTP forward proxy that transparently injects Kerberos (Negotiate) authentication
to upstream proxies or services requiring `Proxy-Authorization: Negotiate <ap req>` like corporate proxies
Cisco Was etc. It is designed for environments where clients are unable or unwilling to handle Kerberos themselves.

The proxy acquires and injects Kerberos tokens using system keytab or the current user session
and relays HTTP requests to the next hop, supporting both regular HTTP requests and HTTP CONNECT tunneling.

## Getting Started

### Dependencies

* Linux (tested on Rocky Linux 9, Ubuntu 24.04)
* Rust and cargo for building
* krb5-libs and krb5-workstation installed (MIT Kerberos)
* Valid Kerberos configuration in `/etc/krb5.conf`
* Valid Kerberos ticket or system keytab for the proxy service

### Installing

Clone this repository and build using Cargo:

```bash
git clone https://github.com/veldrane/krb5proxy.git
cd krb5proxy
make
make install
```

.. or you can use standard cargo tool for building, the resulting binary will be in `target/release/krb5proxy`.


### Executing program

Prepare your environment with a valid Kerberos ticket (e.g., using `kinit`) or ensure the keytab is accessible.

Run the proxy:

```bash
krb5proxy --listen http://127.0.0.1:8080 --proxy http://10.0.0.1:3128 --proxy-hostname proxy.foo.com
```

krb5proxy doesn't resolve ip yet but needs to have a properly configured parrent proxy hostname for getting the right TGS ticket 

Typical usage:
- Configure http_proxy and https_proxy environment variables to point `http://127.0.0.1:8080`
- krb5proxy will forward requests to `10.0.0.1:3128` using Kerberos authentication.

## Help

Make sure your Kerberos environment is properly configured (how to configure kerberos client agains ad will be added soon). 

You can validate your ticket using:

```bash
# klist
Ticket cache: KCM:0
Default principal: student4@CLASS.SYSCALLX86.COM

Valid starting       Expires              Service principal
05/07/2025 20:13:43  05/08/2025 19:31:31  krbtgt/CLASS.SYSCALLX86.COM@CLASS.SYSCALLX86.COM
05/07/2025 20:13:47  05/08/2025 19:31:31  HTTP/proxy.class.syscallx86.com@CLASS.SYSCALLX86.COM
```

If you encounter authentication issues, try renewing or initializing your ticket:

```bash
kinit your-principal@YOUR.REALM
```

Please keep in mind that krb5proxy only forwards an enriched request to the corporate proxy. However, this does not guarantee that you, 
or any ticket holder, have the necessary permission to access the requested page or even communicate with the proxy.

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
## Integration

See [docs/integration.md](docs/integrations.md) for deployment and configuration examples
using Squid, Active Directory, and related infrastructure.

## Authors

[-veldrane](https://github.com/veldrane)

## Version History

* 0.1.5
    * Initial public release

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Inspired by:
* [Cntlm](https://sourceforge.net/projects/cntlm/)

Built with:
* [Rust](https://www.rust-lang.org/)
* [Tokio](https://tokio.rs/)
* [Hyper](https://hyper.rs/)
* [libgssapi](https://github.com/heim-rs/gssapi)
* Wireshark, strace - some hours of debugging ;)
 