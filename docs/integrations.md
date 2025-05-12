# Integration Examples

This document provides integration examples and deployment hints for environments 
using `krb5proxy` together with corporate services like **Squid proxy** and **Active Directory (AD)**.

These examples are provided as *reference only*. 
Always validate the settings against your organization's security policies and infrastructure.

## Table of Contents

- [Squid Proxy Example Krb5/Ldap auth](#squid-proxy-integration)

## Squid Proxy Integration

A typical Squid configuration for handling Kerberos authentication with `krb5proxy` might include authentization part:

```bash
## Kerberos add-on
#auth_param negotiate program /usr/lib64/squid/negotiate_kerberos_auth -k /etc/squid/proxy.keytab

auth_param negotiate program /usr/lib64/squid/negotiate_kerberos_auth -k /etc/squid/proxy.keytab -s HTTP/proxy.class.syscallx86.com@CLASS.SYSCALLX86.COM
```

and authorization part

```bash
external_acl_type ldap_group ttl=60 negative_ttl=10 %LOGIN /usr/lib64/squid/ext_kerberos_ldap_group_acl \
  -g "proxyusers" \
  -b "cn=groups,cn=accounts,dc=class,dc=syscallx86,dc=com" \
  -D "uid=squidbind,cn=users,cn=accounts,dc=class,dc=syscallx86,dc=com" \
  -w "squidbind" \
  -l ldap://idm.class.syscallx86.com

acl allowed_users external ldap_group
http_access allow allowed_users
http_access deny all
```

In this example, any user from the realm class.syscallx86.com (i.e. a Kerberos ticket holder) who is also a member of 
the proxyusers group is allowed to pass through the proxy.


See the provided example in [`contrib/squid/example-squid.conf`](contrib/squid/squid.conf).

