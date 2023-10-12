# CaDiCaL Build

Use `./configure && make` to configure and build `cadical` in the default
`build` sub-directory.

This will also build the library `libcadical.a` as well as the model based
tester `mobical`:
  
    build/cadical
    build/mobical
    build/libcadical.a

The header file of the library is in

    src/cadical.hpp

The build process requires GNU make.  Using the generated `makefile` with
GNU make compiles separate object files, which can be cached (for instance
with `ccache`).  In order to force parallel build you can use the '-j'
option either for 'configure' or with 'make'.  If the environment variable
'MAKEFLAGS' is set, e.g., 'MAKEFLAGS=-j ./configure', the same effect
is achieved and the generated makefile will use those flags.

You might want to check out options of `./configure -h`, such as

    ./configure -c # include assertion checking code

    ./configure -l # include code to really see what the solver is doing

    ./configure -a # both above and in addition `-g` for debugging.

You can easily use multiple build directories, e.g.,

    mkdir debug; cd debug; ../configure -g; make

which compiles and builds a debugging version in the sub-directory `debug`,
since `-g` was specified as parameter to `configure`.  The object files,
the library and the binaries are all independent of those in the default
build directory `build`.

All source files reside in the `src` directory.  The library `libcadical.a`
is compiled from all the `.cpp` files except `cadical.cpp` and
`mobical.cpp`, which provide the applications, i.e., the stand alone solver
`cadical` and the model based tester `mobical`.

If you can not or do not want to rely on our `configure` script nor on our
build system based on GNU `make`, then this is easily doable as follows.

    mkdir build
    cd build
    for f in ../src/*.cpp; do g++ -O3 -DNDEBUG -DNBUILD -c $f; done
    ar rc libcadical.a `ls *.o | grep -v ical.o`
    g++ -O3 -DNDEBUG -DNBUILD -o cadical cadical.o -L. -lcadical
    g++ -O3 -DNDEBUG -DNBUILD -o mobical mobical.o -L. -lcadical

Note that application object files are excluded from the library.
Of course you can use different compilation options as well.
  
Since `build.hpp` is not generated in this flow the `-DNBUILD` flag is
necessary though, which avoids dependency of `version.cpp` on `build.hpp`.
Consequently you will only get very basic version information compiled into
the library and binaries (guaranteed is in essence just the version number
of the library).

Further note that the `configure` script provides some feature checks and
might generate additional compiler flags necessary for compilation.  You
might need to set those yourself or just use a modern C++11 compiler.

This manual build process using object files is fast enough in combination
with caching solutions such as `ccache`.  But it lacks the ability of our
GNU make solution to run compilation in parallel without additional parallel
process scheduling solutions.
