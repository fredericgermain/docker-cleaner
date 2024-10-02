# Support depedencies from bbolt db

https://github.com/etcd-io/bbolt

## Leftover dangling nodes Overlay2

```
/var/lib/docker/buildkit/containerdmeta.db
   v1.buildkit.leases.{overlay2id}
/var/lib/docker/buildkit/metadata_v2.db
   _external/{overlay2id}/buildkit.contenthash.v0
   _main_/{overlay2id}/buildkit.contenthash.v0
   _index/{local.sharedKey}:{overlay2id}
/var/lib/docker/buildkit/snapshots.db
   {overlay2id}/parent
```
