# 🛰️ FAR Agent Kubernetes Controller & Integration with Flintlock

---

## ❓ Why Now?

Autonomous agents like Tiffany require isolated, reproducible, and resource-constrained execution environments — much like microVMs.

Google’s Gemini CLI and other systems assume developer desktops or ephemeral cloud sandboxes. But these models break down when:

* Scaling thousands of agent tasks in parallel
* Providing strict memory/CPU resource limits
* Enforcing strong syscall isolation or jail boundaries
* Running on shared GPU or ARM blade clusters

**MicroVMs**, specifically Firecracker, solve these problems. But orchestrating them at scale requires tooling that understands:

✅ how to create VMs
✅ how to manage agent lifecycles
❌ NOT how to run Kubernetes nodes

---

## 🔥 What is Flintlock?

[LiquidMetal’s Flintlock](https://github.com/liquidmetal-dev/flintlock) is an open-source Firecracker VM provisioning system.

It is:

* 🔧 A low-level control plane that exposes a gRPC API to create/start/stop microVMs
* 🧱 Designed for integration into higher-level platforms
* 💡 Production-tested inside [Weave Ignite](https://github.com/weaveworks/ignite)

Its sibling project, [`cluster-api-provider-microvm`](https://github.com/liquidmetal-dev/cluster-api-provider-microvm), extends Flintlock for use in managing **Kubernetes nodes** as VM-based worker instances.

But that’s exactly where it becomes unsuitable for our needs.

---

## 🚫 Why Not Use `cluster-api-provider-microvm`?

| Feature                       | `cluster-api-provider-microvm` | FAR Requirements                       |
| ----------------------------- | ------------------------------ | -------------------------------------- |
| Assumes each VM is a K8s node | ✅ Yes                          | ❌ No — VMs run agents, not Kubelets    |
| Creates worker node infra     | ✅ Cloud-init, bootstrapping    | ❌ We want agent workloads, not nodes   |
| Built for CAPI                | ✅ Cluster API standard         | ❌ We want independent micro agent pods |
| Requires Kubernetes bootstrap | ✅ kubeadm/cloud-init flows     | ❌ We boot directly to Tiffany agent |

Thus, while Flintlock is ideal as a backend runtime, `cluster-api-provider-microvm` introduces an entire stack of node-management logic we do not want — and cannot reuse.

---

## ✅ What We Will Build

### 🧠 The `far-agent-controller` (Tiffany Native)

A Kubernetes controller that:

* Watches for custom `FarAgent` CRDs
* Calls Flintlock’s gRPC API directly
* Boots minimal rootfs images that run the `tinkerbell` binary
* Tracks state via K8s `status` and `conditions`
* Publishes agent stdout/stderr, metrics, and lifecycle logs

---

### 🎯 `FarAgent` Custom Resource

Example:

```yaml
apiVersion: tinkerbell.dev/v1
kind: FarAgent
metadata:
  name: embedder-worker-01
spec:
  image: ghcr.io/microscaler/tinkerbell-vm:v0.3.1
  memory: 512Mi
  cpu: 1
  entrypoint: /usr/local/bin/tinkerbell
  args: ["--task", "embedder"]
```

Controller responsibilities:

* Translate CRD into Flintlock `CreateVMRequest`
* Start VM and bind agent runtime
* Monitor lifecycle (status, heartbeat, exit code)
* Update `.status.phase = Running | Completed | Failed`

---

## 🤝 Relationship with Flintlock

| Area                          | Flintlock                           | FAR Controller                           |
| ----------------------------- | ----------------------------------- | ---------------------------------------- |
| Firecracker API surface       | ✅ Flintlock abstracts Firecracker   | ✅ Uses Flintlock gRPC for lifecycle ops  |
| VM creation                   | ✅ Flintlock handles it              | ✅ Controller calls it via gRPC           |
| Orchestration & task logic    | ❌ Not supported                     | ✅ Custom controller logic per `FarAgent` |
| Image injection & boot config | ✅ Flintlock supports container boot | ✅ We use it to run `tinkerbell` runtime  |
| Secrets & network             | ❌ Out of scope                      | ✅ Controller injects token/env/volumes   |

We **will contribute patches** to Flintlock as needed, especially around:

* Agent status channels
* Enhanced network config
* VM event streaming
* Minimal API ergonomics for single-process agent bootstraps

---

## 🔐 Security and Isolation

Each `FarAgent` runs:

* In its own microVM (Firecracker)
* With bounded CPU/memory
* With sealed task files or memory modules
* With ephemeral state and no access to host Kubelet, socket, or daemon

This gives us:

* Isolation comparable to containers or Wasm
* Predictable memory/CPU bounds
* Auditable VM lifecycle logs

---

## 🚀 Future Enhancements

* 🔁 Autoscaler for idle vs burst agent pools
* 🧠 VM reuse strategies (warm idle agents)
* 🔐 TPM-backed secrets sealed to agent image
* 📡 Web UI showing real-time agent state on blade clusters
