# Changelog

## 0.1.2

Changed all uses of `Rc` into `Arc`, for multi-threaded use.

## 0.1.1

- Added `From<NodeRef>` implementation for `HashMap<String, String>` that gives
  an easier way to work with the DAG, if you don't care about forwarding
  relationships.

## 0.1.0

- Initial release.
