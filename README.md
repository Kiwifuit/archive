# archive

Safe rust bindings to `libarchive`

*`archive-sys` (and subsiquently this crate and its dependents) require `libarchive` to be installed on the system*

- [archive](#archive)
  - [Development](#development)


I cannot guarantee that this project will be completed, seeing
as I am way too busy these days. But I'll slowly work on it,
perhaps for [DMS](https://github.com/Kiwifuit/DMS) too.

Again, no guarantees.

> [!WARNING]
> This library *does not work*, I'm still trying to work
> out how the library should be used and all the neat bits

## Development
*This crate requires `pkg-config` and `libarchive`'s development files*

To verify, run the following command:
```
$ pkg-config --path libarchive
/usr/lib/x86_64-linux-gnu/pkgconfig/libarchive.pc
```

If `libarchive` does not exist in your machine, install it with this command on Debian and Ubuntu:
```
$ sudo apt install pkgconf libarchive-dev libclang-dev
```