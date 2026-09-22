---
description: A multi-node Internet-sharing chain must be tested in which each node's integrated NIC is its uplink and its USB NIC is its downlink, or stable Ethernet / Optional AP behavior must be tested.
dependencies: [behavior, testing, operating-system]
---

A testing skill: machine-generated, field-level, provisional. It applies the
living's pattern — the built-in port is for upstream and the USB is for
downstream, whatever is plugged where — to a chain of nodes and to stable
Ethernet / Optional AP behavior.

Invariants: integrated NIC is uplink, USB NIC is downlink, unless an accepted
exception names both interfaces and its lifetime; uplink and downlink are
distinct interfaces with stable identity evidence; each downstream link has
one non-overlapping subnet and one DHCP/DNS owner; no upstream/downstream L2
bridge unless the test explicitly targets one; one network manager owns each
link (NetworkManager and networkd never share a link); exactly one durable
firewall owner, and prefer one egress NAT owner at Ouranos, with any
temporary nested NAT named and bounded; missing capability means no gateway
or AP activation; stable Ethernet / Optional AP mode requires an authorized
network-admin action, a transactional health gate, an audit record, and
rollback; WPA-Enterprise acceptance requires unique EAP-TLS client identity,
server certificate validation, revocation behavior, and isolation tests — a
shared password is migration state, never certificate-auth proof.

Required record, per node and link: node identity and observation time;
integrated and USB interface identities, MACs, drivers, and ownership;
declared role (uplink/WAN-facing or downlink/LAN-facing); physical carrier
and identity evidence for the far-end peer; upstream address, gateway,
default route, and DNS source; downstream subnet, DHCP/DNS owner, and lease
observed at the intended peer; forwarding and firewall owner with one
declared egress NAT owner; hop-by-hop ARP/neighbor, route, DNS, and Internet
proof; management-path preservation and rollback state; evidence grade
(source, installed, running, or end-to-end proven). Carrier, a configured
address, or a listening DHCP socket never proves the far peer or Internet; a
historical lease never proves current reachability.

Mandatory cases: full chain (Ouranos integrated uplink -> Ouranos USB
downlink -> Prometheus integrated uplink -> Prometheus USB downlink -> Zeus
integrated uplink); cold boot, USB hotplug, cable unplug/replug, enumeration
change, and DHCP renewal converge to the same roles; missing peer,
carrier-only, stale lease, DHCP-without-offer, and ARP-without-reply stay
failed/unknown, never pass; duplicate DHCP, overlapping subnet, duplicate
NAT, wrong local MAC, ambiguous USB NIC, and network-manager ownership
conflict fail closed; each hop proves local gateway reachability before DNS
and Internet, and the leaf proves the complete return path; unrelated nodes
do not activate from another node's identical payload; stable Ethernet mode
preserves management and a failed health check rolls back; Optional AP stays
absent unless explicitly enabled, and disabling it removes AP-specific
listeners and forwarding without breaking wired transit; EAP-TLS accepts a
valid device certificate and rejects expired, revoked, wrong-EKU, wrong-CA,
and untrusted-server cases; IPv6 cannot bypass the IPv4 forwarding and
isolation policy.

Result language: report **source-published**, **installed-running**, and
**end-to-end-proven** as different grades. Name the exact revision running
versus the latest source. Use `UNKNOWN` where a peer or runtime receipt is
absent. Never infer the whole chain from one successful hop.
