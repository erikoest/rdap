# rdap

A command-line RDAP client for looking up domains, IP addresses, entities, and nameservers.

## Installation

```
cargo install --path .
```

## Usage

```
rdap [OPTIONS] <COMMAND>

Commands:
  domain              Look up a domain name
  host                Look up an IP address or CIDR block
  entity              Look up an entity by handle
  domains             Search for domain names
  entities            Search for entities
  hosts               Search for hosts/nameservers
  help                Fetch server help and usage notices
  nameserver-handle   Look up a nameserver by Norid handle (Norid extension)
  norid-domain-count  Fetch domain count for an identity (Norid extension)

Options:
  -s, --server <URL>         RDAP server base URL (default: https://rdap.org)
      --ipv4                 Force connections over IPv4
      --ipv6                 Force connections over IPv6
  -u, --user <USER>          Username for HTTP Basic authentication
  -p, --password <PASSWORD>  Password for HTTP Basic authentication
      --cursor <CURSOR>      Continue paging from this cursor (RFC-8977)
      --count <N>            Max results per page (RFC-8977)
      --sort <FIELD>         Sort field; prefix with - for descending (RFC-8977)
      --fields <FIELDS>      Comma-separated top-level fields to return (RFC-8982)
  -h, --help                 Print help
```

## Examples

```sh
# Domain registration info
rdap domain example.com

# IP network info
rdap host 8.8.8.8
rdap host 2001:4860:4860::8888

# Entity (registrar, contact) by handle
rdap entity GOGL

# Server help and notices
rdap help

# Search for domains matching a pattern
rdap domains example*

# Search for entities by handle or name
rdap entities --handle ARIN*
rdap entities --fn "Google*"

# Search for nameservers
rdap hosts --name ns1.example*
rdap hosts --ip 192.0.2.0/24

# Paging: limit results and fetch the next page (RFC-8977)
rdap --count 10 domains example*
rdap --count 10 --cursor <cursor-from-previous-response> domains example*

# Partial response: only return specific fields (RFC-8982)
rdap --fields handle,status domain example.com

# Query a specific registry directly
rdap --server https://rdap.verisign.com/com/v1 domain example.com

# Force IPv6 transport
rdap --ipv6 domain example.com

# Authenticated query
rdap --user alice --password secret domain example.com

# Norid extensions (use with --server https://rdap.norid.no)
rdap --server https://rdap.norid.no nameserver-handle NSHA1234-NORID
rdap --server https://rdap.norid.no norid-domain-count <identity>
```

## Notes

- By default queries go to `rdap.org`, which bootstraps to the authoritative registry for the object type.
- `--user` and `--password` must be supplied together.
- `--ipv4` and `--ipv6` are mutually exclusive.
- `entities` search requires `--handle` or `--fn`; `hosts` search requires `--name` or `--ip`.
- `--cursor`, `--count`, `--sort`, and `--fields` apply to all commands but are most relevant to search.
- Norid extensions (`nameserver-handle`, `norid-domain-count`) require `--server https://rdap.norid.no`.
