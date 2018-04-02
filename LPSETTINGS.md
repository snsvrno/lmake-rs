# Settings / Configuration options

## lmake.remove-comments
If set to `true` will automatically remove comments from compiled libraries. Automatically calls the `--remove-comments` switch.

## lmake.name-with-version
If set to `true`, will automatically add the version to the file name of compiled libraries. Automatically calls the `--name-with-version` switch.

## project.library-compile-path
Path to compile libraries into, used when inside a project. If not set it will compile into `.\lib`

## lmake.compile-path
Path to compile a library into, used when not in a project. If not set it will compile into `.\bin`. Not used from a project's toml.