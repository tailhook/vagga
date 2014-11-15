============
Installation
============


Generic Installation
====================


Build-time dependencies:

* Rust_ compiler 0.12 (``rustc``)
* ``make`` (probably gnu variant)
* ``git`` (when installing from git source)

Run-time dependencies (basically none):

* ``glibc``


.. note:: Vagga uses linux_ namespaces, so works on linux system only.


Process is as simple as following::

    git submodule update --init
    make
    sudo make install


.. _Rust: http://rust.org
.. _linux: http://kernel.org


Ubuntu
======

Unfortunately rust-0.12 PPA doesn't work. You need to setup rust from
binaries::

    wget https://static.rust-lang.org/dist/rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
    tar -xf rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
    cd rust-0.12.0-x86_64-unknown-linux-gnu
    ./install.sh --prefix=/usr

Building vagga::

    git clone git://github.com/tailhook/vagga
    cd vagga
    git submodule update --init
    make

Installing::

    sudo make install PREFIX=/usr


