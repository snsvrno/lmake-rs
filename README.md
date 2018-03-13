# Lmake

A lua compilation tool for making a single lua library file from a complex source of multiple files and dependencies.

## Why would I use this?

I made lots of little lua libraries while making projects in [LOVE](https://love2d.org). I always use these to start a project so I pushed them out into their own things, and lots of time they depend on each other. I also wanted each library to be a single .lua file, but that makes it messy to develop. Thus ***LMAKE*** was born.

## What is it?

***LMAKE*** is a command line tool that will take a lua library directory with an appropriate `lib.toml` file and compile it into a single library file. It does this using the lua `package.preload` table to inject sub files into the "requires table" so everything can easily be placed into one file with minimal editing on ***LMAKE***'s part.

## What Can It Do?

Current supported features

- Create a single `lua` file from multiple source files.

Future Supported features

- Include external requirements / dependencies
- Download and manage external requirements / dependencies from online
- Download and manage libraries from online.

## How do I do this?

Just make a lua only library. Then make a `lib.toml` to define it. Here is the simplest `lib.toml`

```toml
name = "stringtools"
user = "snsvrno"
author = "snsvrno <snsvrno@tuta.io>"
version = "1.2.1"

[requires]
_ = "src.tools"
```

The first section is just the information for the library. The listed above are all required.

**Name:** *Required*, The library name

**User:** *Required*, The user or group that "owns" the library

**Author:** *Required*, The username and email of the primary contact for the library

**Version:** *Required*, The library's version

**Upstream:** *Optional*, the url to a git repository that houses the project.

The next section is where you define all source files that make up your library, and where to load them.

```toml
_ = "src.tools"
other = "src.othertools"
```

Here we are loading the contents of the `src.tools` file into the root library file, so the resulting library file will then have all the functions such that `src.tools.AFUNCTION == library.AFUNCTION`.

You can also load files into other parts of the main library. `src.othertools.BFUNCTION == library.other.BFUNCTION`

## Compiling

The simplest is to just be in the library directory and run lmake compile.

```
lmake compile .
```

This will create a new folder `bin` and compile the resulting library there. 

## Testing

There is no testing built into lmake currenty, but I recommend writing unittests and test the compiled library to make sure everything is working and interacting as expected.

## Resulting Code

Here is what a compiled library would look like.

```lua
package.preload['stringtools-253614172587266203315807'] = (function(...)
  local TOOLS = { }

  function TOOLS.split(string,delim)
    -- code here
  end

  function TOOLS.remove(string,characters)
    -- code here
  end

  function TOOLS.removeLeading(string,characters)
    -- code here
  end

  return TOOLS
end)

local library = require ("stringtools-253614172587266203315807")
library.name = 'stringtools'
library.user = 'snsvrno'
library.author = 'snsvrno <snsvrno@tuta.io>'
library.version = '1.2.1'
return library
```

Where the `src.tools` would be 

```lua
local TOOLS = { }

function TOOLS.split(string,delim)
  -- code here
end

function TOOLS.remove(string,characters)
  -- code here
end

function TOOLS.removeLeading(string,characters)
  -- code here
end

return TOOLS
```